use anyhow::{bail, Context, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const API_URL: &str = "https://api.mistral.ai/v1/chat/completions";
const MODELS_URL: &str = "https://api.mistral.ai/v1/models";

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    /// Raciocínio da resposta; reenviado como bloco `thinking` para manter a qualidade nos turnos seguintes.
    #[serde(default)]
    pub thinking: Option<String>,
}

impl Message {
    pub fn new(role: &str, content: impl Into<String>) -> Self {
        Self {
            role: role.to_string(),
            content: content.into(),
            thinking: None,
        }
    }

    pub fn with_thinking(mut self, thinking: Option<String>) -> Self {
        self.thinking = thinking.filter(|t| !t.trim().is_empty());
        self
    }
}

impl Serialize for Message {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Message", 2)?;
        state.serialize_field("role", &self.role)?;
        match self.thinking.as_deref().filter(|_| self.role == "assistant") {
            Some(thinking) => state.serialize_field(
                "content",
                &serde_json::json!([
                    { "type": "thinking", "thinking": [{ "type": "text", "text": thinking }] },
                    { "type": "text", "text": self.content },
                ]),
            )?,
            None => state.serialize_field("content", &self.content)?,
        }
        state.end()
    }
}

/// Nível de raciocínio pedido à API (`reasoning_effort`).
/// Devolve `None` quando o modelo não suporta o parâmetro.
pub fn reasoning_effort(model: &str, think: bool) -> Option<&'static str> {
    let m = model.to_ascii_lowercase();
    match m.as_str() {
        "mistral-large-4" | "mistral-large-4-0" | "zai-glm-5-3" => {
            Some(if think { "high" } else { "low" })
        }
        "mistral-small-latest" | "mistral-small-2603" | "mistral-medium-3-5" => {
            Some(if think { "high" } else { "none" })
        }
        _ => None,
    }
}

pub fn supports_reasoning(model: &str) -> bool {
    reasoning_effort(model, true).is_some()
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [Message],
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_cache_key: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptTokensDetails {
    #[serde(default)]
    pub cached_tokens: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenUsage {
    #[serde(default)]
    pub prompt_tokens: u32,
    #[serde(default)]
    pub completion_tokens: u32,
    #[serde(default)]
    pub total_tokens: u32,
    #[serde(default)]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
}

impl TokenUsage {
    pub fn cached_tokens(&self) -> u32 {
        self.prompt_tokens_details
            .map(|d| d.cached_tokens)
            .unwrap_or(0)
    }
}

#[derive(Deserialize)]
struct StreamChunk {
    #[serde(default)]
    choices: Vec<StreamChoice>,
    #[serde(default)]
    usage: Option<TokenUsage>,
}

#[derive(Deserialize)]
struct StreamChoice {
    delta: Delta,
}

#[derive(Deserialize)]
struct Delta {
    content: Option<serde_json::Value>,
}

/// Separa o `delta.content` em (raciocínio, texto): é string simples ou lista de blocos.
fn split_content(content: &serde_json::Value) -> (String, String) {
    let mut thinking = String::new();
    let mut text = String::new();
    match content {
        serde_json::Value::String(s) => text.push_str(s),
        serde_json::Value::Array(chunks) => {
            for chunk in chunks {
                match chunk.get("type").and_then(|t| t.as_str()) {
                    Some("thinking") => match chunk.get("thinking") {
                        Some(serde_json::Value::String(s)) => thinking.push_str(s),
                        Some(serde_json::Value::Array(parts)) => {
                            for part in parts {
                                if let Some(t) = part.get("text").and_then(|t| t.as_str()) {
                                    thinking.push_str(t);
                                }
                            }
                        }
                        _ => {}
                    },
                    Some("text") => {
                        if let Some(t) = chunk.get("text").and_then(|t| t.as_str()) {
                            text.push_str(t);
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    (thinking, text)
}

#[derive(Debug, Default)]
pub struct ChatOutput {
    pub text: String,
    pub thinking: String,
    pub usage: Option<TokenUsage>,
}

#[derive(Deserialize)]
struct ModelsResponse {
    #[serde(default)]
    data: Vec<ModelEntry>,
}

#[derive(Deserialize)]
struct ModelEntry {
    id: String,
    #[serde(default)]
    capabilities: Option<ModelCapabilities>,
}

#[derive(Deserialize)]
struct ModelCapabilities {
    completion_chat: Option<bool>,
}

pub struct MistralClient {
    http: reqwest::Client,
    api_key: String,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub prompt_cache_key: Option<String>,
    pub reasoning_effort: Option<&'static str>,
}

impl MistralClient {
    pub fn new(api_key: String, model: String) -> Self {
        // Keep the connection pool alive across messages and model changes.
        // Credentials remain on each request, never on the shared client.
        static HTTP: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
        let http = HTTP.get_or_init(|| {
            reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(15))
                .timeout(Duration::from_secs(180))
                .build()
                .unwrap_or_default()
        }).clone();
        Self {
            http,
            api_key,
            model,
            temperature: None,
            max_tokens: None,
            prompt_cache_key: None,
            reasoning_effort: None,
        }
    }

    /// Ativa (`true`) ou desativa o modo "Refletir"; ignorado em modelos sem suporte.
    pub fn with_thinking(mut self, think: bool) -> Self {
        self.reasoning_effort = reasoning_effort(&self.model, think);
        self
    }

    pub fn with_params(mut self, temperature: Option<f32>, max_tokens: Option<u32>) -> Self {
        self.temperature = temperature.map(|t| t.clamp(0.0, 1.5));
        self.max_tokens = max_tokens.filter(|&m| m > 0);
        self
    }

    pub fn with_prompt_cache_key(mut self, cache_key: Option<String>) -> Self {
        self.prompt_cache_key = cache_key
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty());
        self
    }

    /// Envia o histórico, chama `on_token` a cada pedaço de texto recebido
    /// e devolve a resposta completa.
    pub async fn chat_stream(
        &self,
        messages: &[Message],
        on_token: impl FnMut(&str),
    ) -> Result<String> {
        let (full, _) = self
            .chat_stream_cancellable(messages, None, on_token)
            .await?;
        Ok(full)
    }

    /// Como [`Self::chat_stream`], mas interrompe ao ler `cancel`.
    /// Devolve o que já foi recebido até o cancelamento e as métricas de uso de tokens (se informadas pela API).
    pub async fn chat_stream_cancellable(
        &self,
        messages: &[Message],
        cancel: Option<&AtomicBool>,
        on_token: impl FnMut(&str),
    ) -> Result<(String, Option<TokenUsage>)> {
        let out = self
            .chat_stream_full(messages, cancel, on_token, |_| {})
            .await?;
        Ok((out.text, out.usage))
    }

    /// Como [`Self::chat_stream_cancellable`], mas também entrega o raciocínio em `on_thinking`.
    pub async fn chat_stream_full(
        &self,
        messages: &[Message],
        cancel: Option<&AtomicBool>,
        mut on_token: impl FnMut(&str),
        mut on_thinking: impl FnMut(&str),
    ) -> Result<ChatOutput> {
        if self.api_key.trim().is_empty() {
            bail!("informe a chave da API da Mistral nas configuracoes");
        }

        // Blocos `thinking` no histórico só são aceitos com raciocínio ativo.
        let reasoning_on = matches!(self.reasoning_effort, Some(e) if e != "none");
        let stripped: Vec<Message>;
        let messages = if !reasoning_on && messages.iter().any(|m| m.thinking.is_some()) {
            stripped = messages
                .iter()
                .cloned()
                .map(|m| Message { thinking: None, ..m })
                .collect();
            &stripped[..]
        } else {
            messages
        };

        let body = ChatRequest {
            model: &self.model,
            messages,
            stream: true,
            temperature: self
                .temperature
                .map(|t| ((t as f64) * 100.0).round() / 100.0),
            max_tokens: self.max_tokens.filter(|&m| m > 0),
            prompt_cache_key: self.prompt_cache_key.as_deref(),
            reasoning_effort: self.reasoning_effort,
        };

        let resp = self
            .http
            .post(API_URL)
            .bearer_auth(self.api_key.trim())
            .json(&body)
            .send()
            .await
            .context("falha ao conectar na API da Mistral")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            bail!("{}", friendly_error(status.as_u16(), &text));
        }

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();
        let mut full = String::new();
        let mut thinking = String::new();
        let mut usage: Option<TokenUsage> = None;

        'outer: while let Some(chunk) = stream.next().await {
            if cancelled(cancel) {
                break 'outer;
            }
            let chunk = chunk.context("erro ao ler o stream")?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(pos) = buffer.find('\n') {
                let line: String = buffer.drain(..=pos).collect();
                let line = line.trim();
                let Some(data) = line.strip_prefix("data:") else {
                    continue;
                };
                let data = data.trim();
                if data == "[DONE]" {
                    break 'outer;
                }
                if let Ok(parsed) = serde_json::from_str::<StreamChunk>(data) {
                    if let Some(u) = parsed.usage {
                        if u.total_tokens > 0 || u.completion_tokens > 0 || u.prompt_tokens > 0 {
                            usage = Some(u);
                        }
                    }
                    if let Some(content) = parsed
                        .choices
                        .first()
                        .and_then(|c| c.delta.content.as_ref())
                    {
                        let (think, text) = split_content(content);
                        if !think.is_empty() {
                            on_thinking(&think);
                            thinking.push_str(&think);
                        }
                        if !text.is_empty() {
                            on_token(&text);
                            full.push_str(&text);
                        }
                    }
                }
            }
        }
        Ok(ChatOutput {
            text: full,
            thinking,
            usage,
        })
    }

    pub async fn test_connection(&self) -> Result<String> {
        let models = self.list_models().await?;
        Ok(format!(
            "Chave valida, conexao estabelecida ({} modelos disponiveis).",
            models.len()
        ))
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        if self.api_key.trim().is_empty() {
            bail!("informe a chave da API da Mistral");
        }
        let resp = self
            .http
            .get(MODELS_URL)
            .bearer_auth(self.api_key.trim())
            .send()
            .await
            .context("falha ao conectar na API da Mistral")?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            bail!("{}", friendly_error(status.as_u16(), &text));
        }
        let parsed: ModelsResponse = resp
            .json()
            .await
            .context("falha ao interpretar a lista de modelos da Mistral")?;
        let mut ids: Vec<String> = parsed
            .data
            .into_iter()
            .filter(|m| {
                m.capabilities
                    .as_ref()
                    .and_then(|c| c.completion_chat)
                    .unwrap_or(true)
            })
            .map(|m| m.id.trim().to_string())
            .filter(|id| !id.is_empty())
            .collect();
        ids.sort();
        ids.dedup();
        Ok(ids)
    }
}

fn cancelled(cancel: Option<&AtomicBool>) -> bool {
    cancel
        .map(|flag| flag.load(Ordering::Relaxed))
        .unwrap_or(false)
}

fn friendly_error(status: u16, body: &str) -> String {
    let detail = body.trim();
    let detail: String = detail.chars().take(300).collect();
    match status {
        401 => "chave da API invalida ou revogada (401)".to_string(),
        403 => "acesso negado pela API da Mistral (403)".to_string(),
        422 => format!("requisicao invalida (422): {detail}"),
        429 => "limite de uso atingido na Mistral (429). Tente novamente em instantes".to_string(),
        500..=599 => format!("a API da Mistral esta indisponivel ({status})"),
        _ => format!("API da Mistral retornou {status}: {detail}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn friendly_error_messages() {
        assert!(friendly_error(401, "").contains("invalida"));
        assert!(friendly_error(429, "").contains("limite"));
        assert!(friendly_error(503, "").contains("indisponivel"));
        assert!(friendly_error(418, "corpo").contains("418"));
    }

    #[test]
    fn reasoning_effort_by_model() {
        assert_eq!(reasoning_effort("mistral-small-latest", true), Some("high"));
        assert_eq!(reasoning_effort("mistral-small-2603", false), Some("none"));
        assert_eq!(reasoning_effort("mistral-large-latest", false), None);
        assert_eq!(reasoning_effort("mistral-large-latest", true), None);
        assert_eq!(reasoning_effort("mistral-large-4-0", false), Some("low"));
        assert_eq!(reasoning_effort("mistral-medium-3-5", true), Some("high"));
        assert_eq!(reasoning_effort("mistral-medium-2505", true), None);
        assert_eq!(reasoning_effort("open-mistral-nemo", true), None);
    }

    #[test]
    fn delta_content_splits_thinking_and_text() {
        let blocks = serde_json::json!([
            { "type": "thinking", "thinking": [{ "type": "text", "text": "hmm " }] },
            { "type": "text", "text": "10" }
        ]);
        assert_eq!(split_content(&blocks), ("hmm ".to_string(), "10".to_string()));
        assert_eq!(
            split_content(&serde_json::json!("ola")),
            (String::new(), "ola".to_string())
        );
    }

    #[test]
    fn assistant_thinking_is_sent_as_blocks() {
        let msg = Message::new("assistant", "resp").with_thinking(Some("pensei".into()));
        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["content"][0]["type"], "thinking");
        assert_eq!(json["content"][1]["text"], "resp");
        let plain = serde_json::to_value(Message::new("user", "oi")).unwrap();
        assert_eq!(plain["content"], "oi");
    }

    #[test]
    fn message_conversion() {
        let message = Message::new("user", "ola");
        assert_eq!(message.role, "user");
        assert_eq!(message.content, "ola");
    }

    #[test]
    fn stream_chunk_parses_usage() {
        let raw = r#"{"choices":[{"delta":{"content":""}}],"usage":{"prompt_tokens":24,"completion_tokens":86,"total_tokens":110,"prompt_tokens_details":{"cached_tokens":16}}}"#;
        let parsed: StreamChunk = serde_json::from_str(raw).unwrap();
        assert_eq!(
            parsed.usage,
            Some(TokenUsage {
                prompt_tokens: 24,
                completion_tokens: 86,
                total_tokens: 110,
                prompt_tokens_details: Some(PromptTokensDetails { cached_tokens: 16 }),
            })
        );
        assert_eq!(parsed.usage.unwrap().cached_tokens(), 16);
    }

    #[test]
    fn chat_request_serializes_generation_params() {
        let msgs = vec![Message::new("user", "teste")];
        let req = ChatRequest {
            model: "mistral-small-latest",
            messages: &msgs,
            stream: true,
            temperature: Some(0.3),
            max_tokens: Some(1024),
            prompt_cache_key: Some("conv-123"),
            reasoning_effort: Some("high"),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains(r#""temperature":0.3"#));
        assert!(json.contains(r#""max_tokens":1024"#));
        assert!(json.contains(r#""prompt_cache_key":"conv-123""#));

        let req_default = ChatRequest {
            model: "mistral-small-latest",
            messages: &msgs,
            stream: true,
            temperature: None,
            max_tokens: None,
            prompt_cache_key: None,
            reasoning_effort: None,
        };
        let json_default = serde_json::to_string(&req_default).unwrap();
        assert!(!json_default.contains("temperature"));
        assert!(!json_default.contains("max_tokens"));
        assert!(!json_default.contains("prompt_cache_key"));
    }
}

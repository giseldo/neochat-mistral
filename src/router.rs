//! Roteador local de skills: sugere skills para uma mensagem comparando palavras (estilo TF-IDF)
//! com título, tags, descrição e exemplos de cada skill. Não usa rede nem modelos.
//! O catálogo está em inglês; termos comuns em português são expandidos por uma tabela de sinônimos.

use crate::skills::{self, Skill};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Mínimo de caracteres úteis na mensagem para sugerir algo.
const MIN_QUERY_CHARS: usize = 8;
/// Pontuação mínima absoluta para aparecer como sugestão.
const MIN_SCORE: f32 = 2.0;

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub id: &'static str,
    pub title: &'static str,
    pub score: f32,
    /// Termos da mensagem (já normalizados) que casaram.
    pub matched: Vec<String>,
}

/// Termos em português (prefixo normalizado) -> palavras equivalentes no catálogo.
const SYNONYMS: &[(&str, &str)] = &[
    ("resumo", "summary summarization"),
    ("resumi", "summary summarization"),
    ("resuma", "summary summarization"),
    ("sintet", "summary synthesis"),
    ("traduz", "translation localization"),
    ("traduc", "translation localization"),
    ("e-mail", "email"),
    ("email", "email"),
    ("codigo", "code"),
    ("funcao", "code function"),
    ("erro", "error"),
    ("falha", "error"),
    ("reunia", "meeting minutes"),
    ("ata", "meeting minutes"),
    ("pesqui", "research sources"),
    ("planil", "table csv xlsx spreadsheet"),
    ("tabela", "table csv"),
    ("csv", "table csv"),
    ("grafic", "chart caption"),
    ("checkl", "checklist"),
    ("tarefa", "action items todo"),
    ("penden", "action items todo"),
    ("quiz", "quiz"),
    ("prova", "quiz"),
    ("questo", "quiz questions"),
    ("flashc", "flashcards"),
    ("cartoe", "flashcards"),
    ("aula", "lesson plan"),
    ("explic", "explain"),
    ("passo", "step tutor"),
    ("ensina", "tutor"),
    ("reescr", "rewrite clarity"),
    ("clarez", "clarity rewrite"),
    ("tom", "tone"),
    ("formal", "tone"),
    ("titulo", "title headline"),
    ("manche", "headline"),
    ("roteir", "outline"),
    ("esboco", "outline"),
    ("estrut", "outline"),
    ("privac", "privacy redaction pii"),
    ("anonim", "privacy redaction pii"),
    ("pessoai", "privacy pii"),
    ("segura", "safety"),
    ("pergun", "faq question"),
    ("client", "customer support"),
    ("suport", "customer support"),
    ("reclam", "complaint customer"),
    ("post", "social post"),
    ("linked", "linkedin social"),
    ("redes", "social media"),
    ("anunci", "ad copy ads"),
    ("landin", "landing page"),
    ("seo", "seo content brief"),
    ("person", "persona"),
    ("enquet", "survey"),
    ("questi", "survey"),
    ("pesqui", "survey research"),
    ("requis", "requirements"),
    ("priori", "prioritization"),
    ("compar", "comparison competitive"),
    ("concor", "competitive comparison"),
    ("pros", "pros cons"),
    ("contra", "pros cons"),
    ("decisa", "decision matrix"),
    ("sentim", "sentiment"),
    ("feedba", "feedback clustering"),
    ("acessi", "accessibility"),
    ("bug", "bug report"),
    ("versao", "release notes changelog"),
    ("histor", "user story"),
    ("humani", "humanize humanizer ai-tells"),
    ("robot", "humanize ai-tells"),
    ("soa", "humanize voice"),
    ("revisa", "editing rewrite"),
    ("microt", "microcopy"),
    ("tooltip", "microcopy tooltip"),
    ("onboar", "onboarding"),
    ("formul", "form"),
    ("prompt", "prompt"),
    ("citaca", "citations evidence"),
    ("fonte", "sources citations"),
    ("eviden", "evidence"),
    ("proced", "sop workflow"),
    ("proces", "workflow sop"),
    ("execut", "executive brief"),
    ("pontos", "key points extraction"),
    ("extrai", "extraction"),
    ("dados", "data insight"),
    ("docume", "document"),
    ("marca", "brand voice"),
];

const STOPWORDS: &[&str] = &[
    "para", "como", "isso", "esse", "essa", "este", "esta", "uma", "uns", "com", "por", "que",
    "dos", "das", "nos", "nas", "mais", "pode", "voce", "tudo", "quero", "preciso", "favor",
    "the", "and", "for", "with", "this", "that", "from", "into", "your", "you", "are", "not",
    "user", "users", "text", "help", "using", "when", "will", "have", "can", "any", "its",
];

/// Minúsculas, sem acentos.
fn fold(text: &str) -> String {
    text.chars()
        .flat_map(|c| c.to_lowercase())
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            other => other,
        })
        .collect()
}

/// Radical aproximado: prefixo de 6 letras (aproxima "summary"/"summarize" e "resumir"/"resumo").
fn stem(word: &str) -> String {
    word.chars().take(6).collect()
}

fn tokens(text: &str) -> Vec<String> {
    fold(text)
        .split(|c: char| !c.is_alphanumeric() && c != '-')
        .map(|w| w.trim_matches('-'))
        .filter(|w| w.chars().count() >= 3 && !STOPWORDS.contains(w))
        .map(stem)
        .collect()
}

struct Index {
    /// Peso de cada radical em cada skill (mesma ordem de `skills::catalog()`).
    docs: Vec<HashMap<String, f32>>,
    idf: HashMap<String, f32>,
}

fn add_weight(doc: &mut HashMap<String, f32>, text: &str, weight: f32) {
    for token in tokens(text) {
        let entry = doc.entry(token).or_insert(0.0);
        if weight > *entry {
            *entry = weight;
        }
    }
}

fn build_doc(skill: &Skill) -> HashMap<String, f32> {
    let mut doc = HashMap::new();
    add_weight(&mut doc, &skill.title, 3.0);
    add_weight(&mut doc, &skill.id.replace('-', " "), 3.0);
    for tag in &skill.tags {
        add_weight(&mut doc, tag, 3.0);
    }
    add_weight(&mut doc, &skill.description, 1.5);
    for text in skill.activation.use_when.iter().chain(skill.activation.example_queries.iter()) {
        add_weight(&mut doc, text, 1.0);
    }
    doc
}

fn index() -> &'static Index {
    static INDEX: OnceLock<Index> = OnceLock::new();
    INDEX.get_or_init(|| {
        let docs: Vec<HashMap<String, f32>> = skills::catalog().iter().map(build_doc).collect();
        let n = docs.len().max(1) as f32;
        let mut df: HashMap<String, f32> = HashMap::new();
        for doc in &docs {
            for token in doc.keys() {
                *df.entry(token.clone()).or_insert(0.0) += 1.0;
            }
        }
        let idf = df.into_iter().map(|(t, d)| (t, (1.0 + n / d).ln())).collect();
        Index { docs, idf }
    })
}

/// Radicais da mensagem, incluindo equivalentes em inglês dos termos em português.
fn query_terms(text: &str) -> Vec<String> {
    let mut terms = tokens(text);
    let mut extra = Vec::new();
    for term in &terms {
        for (prefix, english) in SYNONYMS {
            if term.starts_with(prefix) || (prefix.len() > term.len() && prefix.starts_with(term.as_str())) {
                extra.extend(tokens(english));
            }
        }
    }
    terms.extend(extra);
    terms.sort();
    terms.dedup();
    terms
}

/// Até `limit` skills relevantes para `text`, ignorando as já ativas.
pub fn suggest(text: &str, active: &[String], limit: usize) -> Vec<Suggestion> {
    if text.trim().chars().count() < MIN_QUERY_CHARS {
        return Vec::new();
    }
    let terms = query_terms(text);
    if terms.is_empty() {
        return Vec::new();
    }
    let index = index();
    let mut scored: Vec<Suggestion> = skills::catalog()
        .iter()
        .zip(&index.docs)
        .filter(|(skill, _)| !active.contains(&skill.id))
        .filter_map(|(skill, doc)| {
            let mut score = 0.0;
            let mut matched = Vec::new();
            for term in &terms {
                if let Some(weight) = doc.get(term) {
                    score += weight * index.idf.get(term).copied().unwrap_or(1.0);
                    matched.push(term.clone());
                }
            }
            (score >= MIN_SCORE).then(|| Suggestion {
                id: &skill.id,
                title: &skill.title,
                score,
                matched,
            })
        })
        .collect();
    scored.sort_by(|a, b| b.score.total_cmp(&a.score));
    let best = scored.first().map_or(0.0, |s| s.score);
    scored.retain(|s| s.score >= best * 0.6);
    scored.truncate(limit);
    scored
}

#[cfg(test)]
mod tests {
    use super::*;

    fn top(text: &str) -> Vec<&'static str> {
        suggest(text, &[], 3).into_iter().map(|s| s.id).collect()
    }

    #[test]
    fn suggests_for_portuguese_requests() {
        assert!(top("Resuma este artigo para mim, por favor").contains(&"concise-summary"));
        assert!(top("Traduza esse texto para o inglês").contains(&"translation-localization"));
        assert!(top("Faça um quiz sobre história do Brasil").contains(&"quiz-generator"));
        assert!(top("Monte flashcards da matéria de biologia").contains(&"flashcard-maker"));
        assert!(top("Escreva um e-mail pedindo reembolso ao fornecedor").contains(&"email-draft"));
        assert!(top("Gere a ata da reunião de ontem").contains(&"meeting-minutes"));
    }

    #[test]
    fn suggests_for_english_requests() {
        assert!(top("Write release notes for the new version").contains(&"release-notes"));
        assert!(top("Create a persona for our app users").contains(&"persona-builder"));
    }

    #[test]
    fn ai_sounding_text_points_to_writing_editors() {
        let ids = top("Este texto soa como escrito por IA, deixe mais humano");
        assert!(ids.contains(&"humanizer") || ids.contains(&"no-ai-slop"), "{ids:?}");
    }

    #[test]
    fn small_talk_and_short_text_get_no_suggestion() {
        assert!(top("oi, tudo bem?").is_empty());
        assert!(top("ok").is_empty());
        assert!(top("").is_empty());
    }

    #[test]
    fn active_skills_are_not_suggested_again() {
        let active = vec!["concise-summary".to_string()];
        let ids: Vec<&str> = suggest("Resuma este artigo para mim", &active, 3).into_iter().map(|s| s.id).collect();
        assert!(!ids.contains(&"concise-summary"));
    }

    #[test]
    fn results_are_limited_and_sorted() {
        let found = suggest("Resuma e traduza esta reunião em tópicos e checklist", &[], 2);
        assert!(found.len() <= 2);
        assert!(found.windows(2).all(|w| w[0].score >= w[1].score));
    }
}

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Fluxo de trabalho

Sempre que o usuário pedir uma entrega, faça **commit, push e release** (nesta ordem):

1. Atualize `version` em `Cargo.toml` (patch para correções/refatorações, minor para funcionalidades) e compile.
2. Commit na `master` (mensagens em português, estilo `feat:`/`fix:`/`refactor:`), crie a tag `vX.Y.Z` e faça `git push origin master vX.Y.Z`.
3. Crie a release com `gh release create vX.Y.Z --title "NeoChat vX.Y.Z" --notes "..."`.
4. O executável não é anexado localmente: `.github/workflows/release.yml` roda `cargo build --release` no GitHub Actions ao receber a tag `v*` e anexa `NeoChat.exe` e o `.zip` (leva ~6 min). Confira com `gh run list`.
5. Gere também o `dist` local com `.\scripts\package.ps1` (feche o app antes, senão o build falha com "Acesso negado"); `dist\` fica fora do git.
6. "cargo release" aqui significa `cargo build --release` / `cargo run --release` (não há a ferramenta `cargo-release`); o binário local fica em `target\release\neochat.exe`.

## Comandos

App desktop para Windows (Rust, egui/eframe). Requer toolchain MSVC e Windows SDK (`rc.exe`, usado por `build.rs`).

```bash
cargo run --release                        # app gráfico
cargo run --release --bin neochat-cli      # versão de terminal (MISTRAL_API_KEY, MISTRAL_MODEL)
cargo test                                 # todos os testes (unitários, dentro dos módulos)
cargo test reasoning_effort_by_model       # um teste específico
.\scripts\package.ps1                      # gera dist\NeoChat.exe
```

Não há lint/CI além do workflow de release.

## Arquitetura

Crate `neochat` com `lib.rs` (módulos `app`, `mistral`, `oauth`, `secrets`, `storage`, `theme`) e dois binários: `src/main.rs` (GUI) e `src/bin/neochat-cli.rs`. Estado persistido em `%APPDATA%\NeoChat` (`config.json`, `conversations.json`, `secret.dat` criptografado com DPAPI).

- **`mistral.rs`** — cliente HTTP da API da Mistral (`chat/completions` em streaming SSE e `/v1/models`). `chat_stream_full` é o núcleo; `chat_stream_cancellable` e `chat_stream` delegam a ele. O `delta.content` pode ser string ou lista de blocos (`thinking`/`text`), tratado em `split_content`. `Message` tem `Serialize` manual: mensagens do assistente com `thinking` viram lista de blocos, e o cliente remove esse raciocínio do histórico quando o raciocínio não está ativo (a API devolve 400 caso contrário).
- **Modos Rápido/Refletir** — `reasoning_effort(model, think)` mapeia o nome do modelo (substring `small`/`medium`/`large`/`glm`) para o parâmetro `reasoning_effort` da API; modelos sem suporte retornam `None` e não recebem o parâmetro (e o seletor fica oculto no compositor). A escolha é `Config.think`.
- **`storage.rs`** — `Config`, `StoredMessage`, conversas e `Store` (carrega/salva JSON). Campos novos em `Config`/`StoredMessage` precisam de `#[serde(default)]` para não quebrar arquivos antigos. `StoredMessage::to_api` converte para `mistral::Message`.
- **`app/`** — UI egui. `mod.rs` guarda o `NeoChat` (estado, envio, `poll_events`); a requisição roda em um runtime tokio e se comunica com a UI por canal `mpsc` com `Event::{Token, Thinking, Done, Error}`, acumulando em `partial`/`partial_thinking` até `push_assistant`. `chat.rs` renderiza mensagens (Markdown via egui_commonmark), `composer.rs` o campo de entrada com seletores de modelo e modo, `settings.rs` as configurações, `sidebar.rs` as conversas, `i18n.rs` a tradução (português é a chave; `tr("texto pt")` devolve en/fr; textos novos de UI precisam entrar no `match`).
- **`oauth.rs`** — login com conta Mistral (PKCE); **`secrets.rs`** — DPAPI via `windows-sys`.

Os nomes de modelos em `storage::MODELS` usam aliases `-latest`; a lista completa vem de `/v1/models` pelo botão "Carregar todos os modelos".

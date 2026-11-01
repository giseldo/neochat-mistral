# NeoChat

[Leia em português](README.md)

A Windows desktop client written in Rust for chatting with models available through the [Mistral](https://mistral.ai) API. It provides streaming responses, Markdown and local conversation history. The interface uses egui and eframe.

> **Independent project:** NeoChat has no affiliation or relationship with Mistral AI. It is not an official Mistral product.

## Features

- Streaming responses with stop, copy and regenerate actions.
- Sidebar conversation management, including title search, creation, renaming and deletion.
- Model selection in the top bar or settings, with an option to load models available from the API.
- Temperature, token limit, context caching and system, light or dark theme settings.
- Mistral account login or API key authentication, with the key protected by Windows DPAPI.
- Generation metrics when available and a `neochat-cli` terminal client.

## Build requirements

- 64-bit Windows 10 or 11 with an up-to-date graphics driver.
- Stable Rust with the MSVC toolchain (tested with Rust 1.99.1).
- Visual Studio Build Tools with the **Desktop development with C++** workload, including the Windows SDK and `rc.exe`.

## Run

```powershell
cargo run --release
```

Open **Configurações** (Settings) in the sidebar or press `Ctrl+,`. In the **Conta** (Account) tab, sign in to your Mistral account or enter a key from the [Mistral console](https://console.mistral.ai/api-keys). Click **Salvar** (Save) to apply edited settings. The application interface is in Portuguese.

Alternatively, provide the key through an environment variable:

```powershell
$env:MISTRAL_API_KEY = "your-api-key"
cargo run --release
```

Settings are organized into Account, Model, Generation, Appearance and About tabs. The connection test is in the Account tab.

## Package

```powershell
.\scripts\package.ps1
```

The script builds the release and creates `dist\NeoChat.exe`, a copy named `dist\neochat-desktop-mistral.exe`, and `dist\README.md`. The folder can be copied to another compatible Windows computer; Rust and build tools are not needed to run the package.

## Terminal client

```powershell
$env:MISTRAL_API_KEY = "your-api-key"
cargo run --release --bin neochat-cli
```

The CLI also uses the key saved by the desktop application if the variable is not set. Set `MISTRAL_MODEL` to select a model. Available commands are `/ajuda` (help), `/limpar` (clear), `/modelo` (current model) and `/sair` (exit).

## Keyboard shortcuts

| Shortcut | Action |
| --- | --- |
| `Enter` | Send message |
| `Shift+Enter` | Insert line break |
| `Esc` | Stop generation or close window |
| `Ctrl+N` | New conversation |
| `Ctrl+,` | Open settings |
| `Ctrl+B` | Show or hide the sidebar |

## Data and privacy

The application stores data in `%APPDATA%\NeoChat`:

| File | Contents |
| --- | --- |
| `config.json` | Model, theme, generation parameters and preferences |
| `conversations.json` | Conversations, messages and metrics |
| `secret.dat` | API key encrypted with DPAPI for the current Windows user |

Messages you send are transmitted to the Mistral API. Login and model loading also communicate with Mistral services.

## Structure and verification

The UI is in `src/app/`; the API client is in `src/mistral.rs`; persistence is in `src/storage.rs` and `src/secrets.rs`; and the CLI is in `src/bin/neochat-cli.rs`.

```powershell
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

## Development

Developed by Giseldo.

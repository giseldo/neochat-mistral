# NeoChat

[Versão em Português](README.md)

A fast, lightweight native Windows desktop client written in Rust for interacting with [Mistral AI](https://mistral.ai) models. Built with `egui` and `eframe` (rendered via `wgpu` and Direct3D 12), NeoChat features real-time streaming, Markdown rendering, conversation history, prompt caching, generation metrics, DPAPI credential encryption, and a companion CLI tool.

---

## Features

- **Real-time Streaming**: Token-by-token response streaming with an instant **Stop** button (or `Esc`).
- **Rich Markdown Rendering**: Full formatting for code blocks, syntax highlighting, lists, and tables via `egui_commonmark`, with one-click **Copy** and **Regenerate** actions.
- **Generation & Performance Metrics**: Displays output token count, cached tokens, response time, and generation throughput (tokens/second) beneath assistant messages.
- **Prompt Caching Support**: Toggleable prompt caching reusing conversation context for up to 90% discount on cached input tokens.
- **Dynamic Model Selection**: Switch models on the fly from the top bar or settings; easily query and load the full catalog of available models directly from Mistral's API.
- **Customizable Generation Parameters**:
  - **Temperature**: Fine-tune creativity from `0.00` (deterministic/coding) to `1.50` (creative).
  - **Max Tokens**: Set custom response limits or use presets (`512`, `2048`, `4096`, `Auto`).
- **Sidebar & Conversation Management**: Organize conversations in the sidebar with create, rename (right-click context menu), and delete capabilities.
- **Theme & Windows Integration**: System, Light, or Dark themes with synchronized native Windows 10/11 title bar and custom application icon.
- **Secure Key Storage**: API key is encrypted using Windows DPAPI (`secret.dat`), restricted exclusively to the current Windows user. Also supports logging in via browser with your Mistral account.
- **Keyboard Shortcuts**:
  - `Enter`: Send message
  - `Shift + Enter`: Insert line break
  - `Esc`: Cancel streaming / close modal
  - `Ctrl + N`: New conversation
  - `Ctrl + ,`: Open Settings
- **Terminal Version (`neochat-cli`)**: Standalone command-line interface sharing the desktop app's credentials and storage.

---

## Requirements

- **Operating System**: Windows 10 / 11 x64.
- **Rust**: Stable Rust (tested with 1.99+) with the `x86_64-pc-windows-msvc` toolchain.
- **Visual Studio Build Tools**: "Desktop development with C++" workload installed (linker, Windows SDK, and `rc.exe` for Windows resource compilation).
- **Graphics**: GPU with updated drivers (hardware-accelerated rendering via `wgpu` / Direct3D 12).

---

## Getting Started

### 1. Run via Cargo

```powershell
cargo run --release
```

On first launch, open **Settings** (gear icon in the top right or `Ctrl + ,`) and paste your API key from [Mistral Console](https://console.mistral.ai/api-keys), or log in through your browser.

Alternatively, provide the API key using an environment variable:

```powershell
$env:MISTRAL_API_KEY = "your-api-key"
cargo run --release
```

---

## Building the Executable

To package a standalone release executable:

```powershell
.\scripts\package.ps1
```

This compiles the release binary and packages `dist\NeoChat.exe` along with `README.md`. The portable `dist` folder can be copied and run on any compatible Windows PC without requiring Rust or build tools installed.

---

## Terminal Version (CLI)

NeoChat includes a lightweight CLI binary for quick terminal conversations:

```powershell
$env:MISTRAL_API_KEY = "your-api-key"
cargo run --release --bin neochat-cli
```

### CLI Commands:
- `/ajuda`: Show available commands
- `/limpar`: Clear current conversation history
- `/modelo`: Display active model
- `/sair`: Exit program

> **Note:** If `MISTRAL_API_KEY` is not set in the environment, the CLI automatically loads the encrypted key saved by the desktop application. You can also specify a model via `$env:MISTRAL_MODEL = "mistral-large-latest"`.

---

## Data & Privacy

All application data is stored locally in `%APPDATA%\NeoChat`:

| File | Content |
| --- | --- |
| `config.json` | Selected model, theme preference, window size, generation parameters, prompt caching status |
| `conversations.json` | Chat threads, message history, timestamps, and performance metrics |
| `secret.dat` | API key encrypted using Windows DPAPI (CryptProtectData for current user) |

Your messages and data are stored strictly on your local machine and are only transmitted to Mistral AI's official endpoints (`api.mistral.ai`) when sending prompts or querying model lists.

---

## Project Structure

```
src/
  main.rs             Application entry point and eframe window setup
  bin/neochat-cli.rs  Standalone CLI interface
  app/                UI components (sidebar, chat area, composer, settings modal)
  mistral.rs          HTTP client, streaming parser, and Mistral API integration
  secrets.rs          Windows DPAPI encryption/decryption routines
  storage.rs          JSON persistence for configurations, messages, and metrics
  theme.rs            Colors, typography, and styling themes
assets/               Icon (.ico, .png) and application manifest
build.rs              Windows resource script compilation (icon embedding)
scripts/              Packaging and icon generation automation scripts
```

---

## Testing & Quality Assurance

Run test suites and linter checks:

```powershell
# Run unit and integration tests
cargo test --all-targets

# Run Clippy checks with zero warnings tolerance
cargo clippy --all-targets -- -D warnings
```

---

## License

This project is open-source. See the repository for license details.

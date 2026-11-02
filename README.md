# NeoChat

[Read in English](README.en.md)

<img width="1919" height="1007" alt="image" src="https://github.com/user-attachments/assets/70c91454-0fe3-4520-897b-6267b3ee409a" />

<img width="1914" height="1003" alt="image" src="https://github.com/user-attachments/assets/0fb6caf7-64da-4a82-9054-10bc9d5a96aa" />

Cliente desktop para Windows, desenvolvido em Rust, para conversar com modelos disponíveis pela API da [Mistral](https://mistral.ai). Oferece respostas em streaming, Markdown e histórico local de conversas. A interface usa egui e eframe.

> **Projeto independente:** o NeoChat não tem vínculo, afiliação ou relação com a empresa Mistral AI. Não é um produto oficial da Mistral.

## Funcionalidades

- Respostas em streaming, com opção de interromper, copiar ou regenerar.
- Conversas na barra lateral, com busca por título, criação, renomeação e exclusão.
- Seleção de modelo na barra superior ou nas configurações, com opção de carregar os modelos disponíveis na API.
- Ajustes de temperatura, limite de tokens, cache de contexto e tema claro, escuro ou do sistema.
- Login pela conta Mistral ou uso de chave de API, protegida no Windows com DPAPI.
- Métricas de geração quando disponíveis e cliente de terminal `neochat-cli`.

## Requisitos para compilar

- Windows 10 ou 11, 64 bits, com driver gráfico atualizado.
- Rust estável com toolchain MSVC (testado com Rust 1.99.1).
- Visual Studio Build Tools com a carga de trabalho **Desktop development with C++**, incluindo Windows SDK e `rc.exe`.

## Executar

```powershell
cargo run --release
```

Abra **Configurações** na barra lateral ou pressione `Ctrl+,`. Na aba **Conta**, entre com a conta Mistral ou informe uma chave obtida no [console da Mistral](https://console.mistral.ai/api-keys). Para confirmar as configurações editadas, clique em **Salvar**.

Também é possível fornecer a chave pela variável de ambiente:

```powershell
$env:MISTRAL_API_KEY = "sua-chave"
cargo run --release
```

As configurações são organizadas nas abas **Conta**, **Modelo**, **Geração**, **Aparência** e **Sobre**. O botão **Testar conexão** está na aba Conta.

## Gerar o pacote

```powershell
.\scripts\package.ps1
```

O script compila em modo release e gera `dist\NeoChat.exe`, uma cópia com o nome `dist\NeoChat Mistral.exe` e `dist\README.md`. A pasta pode ser copiada para outro computador Windows compatível; Rust e as ferramentas de compilação não são necessários para executar o pacote.

## Versão de terminal

```powershell
$env:MISTRAL_API_KEY = "sua-chave"
cargo run --release --bin neochat-cli
```

A CLI também usa a chave salva pelo aplicativo se a variável não estiver definida. O modelo pode ser definido com `MISTRAL_MODEL`. Os comandos disponíveis são `/ajuda`, `/limpar`, `/modelo` e `/sair`.

## Atalhos

| Atalho | Ação |
| --- | --- |
| `Enter` | Enviar mensagem |
| `Shift+Enter` | Inserir quebra de linha |
| `Esc` | Interromper geração ou fechar janela |
| `Ctrl+N` | Nova conversa |
| `Ctrl+,` | Abrir configurações |
| `Ctrl+B` | Mostrar ou ocultar a barra lateral |

## Dados e privacidade

O aplicativo armazena dados em `%APPDATA%\NeoChat`:

| Arquivo | Conteúdo |
| --- | --- |
| `config.json` | Modelo, tema, parâmetros de geração e preferências |
| `conversations.json` | Conversas, mensagens e métricas |
| `secret.dat` | Chave de API criptografada com DPAPI para o usuário atual do Windows |

As mensagens enviadas são transmitidas à API da Mistral. O login e o carregamento de modelos também se comunicam com serviços da Mistral.

## Estrutura e verificação

A interface fica em `src/app/`; o cliente da API em `src/mistral.rs`; a persistência em `src/storage.rs` e `src/secrets.rs`; e a CLI em `src/bin/neochat-cli.rs`.

```powershell
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

## Desenvolvimento

Desenvolvido por Giseldo.

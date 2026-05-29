<p align="center">
  <img src="src-tauri/icons/icon.png" alt="Recall" width="128">
</p>

<h1 align="center">Recall</h1>

<p align="center">
  <strong>Captura e organizacao pessoal direto da system tray.<br>Abre, anota, some.</strong>
</p>

<p align="center">
  <a href="#features">Features</a> &bull;
  <a href="#stack">Stack</a> &bull;
  <a href="#getting-started">Getting Started</a> &bull;
  <a href="#shortcuts">Shortcuts</a> &bull;
  <a href="#license">License</a>
</p>

---

## Features

| Feature | Descricao |
|---------|-----------|
| **Editor WYSIWYG** | Negrito, italico, headings, listas, citacoes, links, imagens |
| **System tray** | Abre com atalho global, fecha com Escape, icone na barra de tarefas |
| **Captura rapida** | `Ctrl+Shift+N` — nota temporaria que expira em 24h |
| **Lembretes** | Recorrencia diaria/semanal/mensal, notificacoes nativas |
| **Templates** | Reuniao, Tarefa, Diario, Estudo + templates customizados |
| **Historico de versoes** | Ate 20 snapshots por nota, restaurar qualquer versao |
| **Categorias e tags** | Filtro por categoria/tag, drag-and-drop para reordenar |
| **Imagem do clipboard** | Cole screenshots direto no editor |
| **Import/export** | Backup JSON completo com validacao e deteccao de conflitos |
| **Temas** | Dark e light com tamanho de fonte personalizavel |
| **Atalhos configuraveis** | Dois atalhos globais (abrir app + nova nota) |
| **Pin mode** | Janela sempre visivel (always-on-top) |

## Stack

| Camada | Tecnologia |
|--------|------------|
| Runtime | [Tauri 2.x](https://v2.tauri.app/) |
| Backend | Rust (serde, chrono, uuid, base64) |
| Frontend | Vanilla JS (ES modules, zero bundler) |
| Armazenamento | JSON local (um arquivo por nota/lembrete) |
| Icones | SVG inline ([Lucide](https://lucide.dev/)) |
| Testes | 190+ Rust (unit + integration) + 28 Vitest |

**Zero dependencias JS em runtime.**

## Requirements

| Componente | Versao |
|------------|--------|
| Rust | 1.70+ |
| Node.js | 18+ (apenas para testes) |
| OS | Windows 10/11 |

## Getting Started

```bash
# Instalar Tauri CLI
cargo install tauri-cli

# Modo desenvolvimento
cargo tauri dev

# Gerar instalador
cargo tauri build
```

O app inicia minimizado na system tray. Clique no icone da tray ou pressione `Ctrl+Alt+X` para abrir.

## Shortcuts

| Atalho | Acao |
|--------|------|
| `Ctrl+Alt+X` | Abrir/fechar janela |
| `Ctrl+N` | Nova nota |
| `Ctrl+Shift+N` | Captura rapida (24h) |
| `Ctrl+P` | Buscar |
| `Ctrl+S` | Salvar nota |
| `Ctrl+,` | Configuracoes |
| `Escape` | Fechar janela |

## Project Structure

```
src-tauri/src/
  main.rs          # Bootstrap, plugins, IPC
  commands.rs      # 30+ comandos IPC (toda a API)
  models.rs        # Structs, DTOs, filtros
  storage.rs       # Persistencia JSON (atomic writes)
  cache.rs         # Cache in-memory com invalidacao lazy
  scheduler.rs     # Polling de lembretes em background
  shortcuts.rs     # Parsing/registro de atalhos globais
  tray.rs          # Icone da tray e menu de contexto
  window.rs        # Show/toggle/focus da janela

src/
  index.html       # Shell single-page
  scripts/
    app.js         # Entry point, tabs, atalhos
    notes.js       # Lista, busca, filtros, drag-drop
    editor.js      # WYSIWYG, toolbar, historico
    reminders.js   # Lista, calendario
    settings.js    # Tema, atalhos, fonte, backup
    utils.js       # Markdown, toasts, sanitizacao
    icons.js       # Biblioteca de icones Lucide
    api.js         # Bridge IPC Tauri
  styles/
    base.css       # Reset, layout, scrollbar
    themes.css     # Variaveis CSS dark/light
    components.css # Cards, botoes, editor, calendario, modais
```

## Build

```bash
cargo tauri build
```

Gera o instalador NSIS em `src-tauri/target/release/bundle/nsis/`.

## License

MIT

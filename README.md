# Recall

Captura e organizacao pessoal direto da system tray. Abre, anota, some.

## O que faz

- Notas com editor WYSIWYG e markdown
- Lembretes com calendario e recorrencia
- Templates personalizaveis
- Import/export de dados
- Notas temporarias (24h)
- Tudo local, tudo offline

## Stack

- **Backend:** Rust + Tauri 2.x
- **Frontend:** Vanilla JS, HTML, CSS
- **Armazenamento:** JSON local com atomic writes
- **Testes:** 190+ Rust + 28 frontend

## Rodando

```bash
cargo tauri dev
```

## Build

```bash
cargo tauri build
```

Gera um instalador `.msi` em `src-tauri/target/release/bundle/`.

## Atalhos

| Atalho | Acao |
|--------|------|
| Ctrl+Shift+N | Nota rapida (temporaria, 24h) |
| Ctrl+N | Nova nota |
| Ctrl+P | Buscar |
| Ctrl+, | Configuracoes |
| Escape | Fechar janela |

## Licenca

MIT

# Recall - Design Specification

**Data:** 2026-05-27
**Status:** Aprovado para implementacao

---

## 1. Visao Geral

Recall e um aplicativo Windows leve de organizacao pessoal que roda em segundo plano na bandeja do sistema. Permite criar notas rapidas, definir lembretes com notificacoes nativas, e acessar tudo via atalho global ou icone na tray.

**Objetivos:**
- Minimo consumo de memoria e recursos
- Acesso instantaneo via atalho global ou tray
- Notas cotidianas + lembretes com notificacoes Windows
- Instalacao simples (wizard) e auto-atualizacao via GitHub
- Iniciar junto com o Windows

---

## 2. Tecnologias

| Componente | Tecnologia |
|---|---|
| Framework | Tauri 2.x |
| Backend | Rust |
| Frontend | HTML + CSS + JavaScript (vanilla) |
| Armazenamento | Arquivos JSON (um por entidade) |
| UI Engine | WebView2 (embutido no Windows 10/11) |
| Auto-start | `tauri-plugin-autostart` |
| Auto-update | `tauri-plugin-updater` (GitHub Releases) |
| Instalador | WiX (.msi) gerado pelo Tauri |
| Notificacoes | `tauri-plugin-notification` |

**Nao utilizamos:** React, TypeScript, bundlers, ou qualquer framework frontend. HTML/CSS/JS puro para manter simples e leve.

---

## 3. Arquitetura

```
Recall/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs              # Entry point, setup do Tauri
│   │   ├── commands.rs          # Comandos IPC expostos ao frontend
│   │   ├── storage.rs           # Leitura/escrita de arquivos JSON
│   │   ├── scheduler.rs         # Timer de lembretes em background
│   │   ├── tray.rs              # System tray e menu de contexto
│   │   └── shortcuts.rs         # Atalho global configuravel
│   ├── data/                    # Pasta de dados do usuario
│   │   ├── notes/               # Uma nota = um arquivo .json
│   │   ├── reminders/           # Um lembrete = um arquivo .json
│   │   └── config.json          # Configuracoes globais
│   ├── icons/                   # Icones do app (tray, instalador)
│   ├── tauri.conf.json          # Configuracao do Tauri
│   └── Cargo.toml
├── src/
│   ├── index.html               # Pagina principal
│   ├── styles/
│   │   ├── base.css             # Reset e variaveis CSS
│   │   ├── dark.css             # Tema escuro
│   │   ├── light.css            # Tema claro
│   │   └── components.css       # Estilos dos componentes
│   └── scripts/
│       ├── app.js               # Inicializacao e roteamento
│       ├── api.js               # Wrapper das chamadas invoke()
│       ├── notes.js             # CRUD e renderizacao de notas
│       ├── reminders.js         # CRUD e renderizacao de lembretes
│       ├── settings.js          # Tela de configuracoes
│       └── utils.js             # Funcoes utilitarias
├── docs/
│   └── superpowers/specs/       # Este documento
└── package.json                 # Dependencias do frontend (se necessario)
```

**Fluxo de comunicacao:**
```
Frontend (JS)  --invoke()-->  Rust Commands  -->  Storage (JSON)
                                   |
                                   +-->  Scheduler (timer)
                                   +-->  Notifications (Windows)
                                   +-->  Tray (system tray)
```

Frontend nunca acessa disco diretamente. Toda operacao passa pelos comandos Rust via IPC (`invoke`).

---

## 4. Modelo de Dados

### 4.1 Nota

**Localizacao:** `data/notes/{id}.json`

```json
{
  "id": "a1b2c3d4-uuid",
  "title": "Titulo da nota",
  "content": "Corpo da nota em texto simples",
  "category": "Trabalho",
  "tags": ["urgente", "reuniao"],
  "pinned": false,
  "created_at": "2026-05-27T14:00:00Z",
  "updated_at": "2026-05-27T14:30:00Z"
}
```

| Campo | Tipo | Obrigatorio | Descricao |
|---|---|---|---|
| id | string (UUID) | Sim | Identificador unico |
| title | string | Sim | Titulo da nota |
| content | string | Nao | Corpo da nota |
| category | string | Nao | Categoria (ex: Trabalho, Pessoal) |
| tags | string[] | Nao | Lista de tags |
| pinned | boolean | Nao | Fixada no topo da lista |
| created_at | ISO 8601 | Sim | Data de criacao |
| updated_at | ISO 8601 | Sim | Ultima atualizacao |

### 4.2 Lembrete

**Localizacao:** `data/reminders/{id}.json`

```json
{
  "id": "e5f6g7h8-uuid",
  "title": "Pagar conta de luz",
  "description": "Vence dia 30",
  "note_id": null,
  "trigger_at": "2026-05-28T10:00:00Z",
  "repeat": null,
  "relative_minutes": null,
  "status": "pending",
  "created_at": "2026-05-27T14:00:00Z"
}
```

| Campo | Tipo | Obrigatorio | Descricao |
|---|---|---|---|
| id | string (UUID) | Sim | Identificador unico |
| title | string | Sim | Titulo do lembrete |
| description | string | Nao | Detalhes adicionais |
| note_id | string | Nao | ID de nota vinculada |
| trigger_at | ISO 8601 | Sim | Momento do disparo |
| repeat | string/null | Nao | `"daily"` / `"weekly"` / `"monthly"` |
| relative_minutes | number/null | Nao | Minutos relativos (preenchido na criacao) |
| status | string | Sim | `"pending"` / `"fired"` / `"dismissed"` |
| created_at | ISO 8601 | Sim | Data de criacao |

**Tipos de lembrete:**
- **Data/hora exata:** `trigger_at` preenchido, `repeat` e `relative_minutes` nulos
- **Recorrencia:** `trigger_at` = primeiro disparo, `repeat` preenchido
- **Timer relativo:** `relative_minutes` preenchido na criacao, `trigger_at` calculado como `now + relative_minutes`

### 4.3 Configuracoes

**Localizacao:** `data/config.json`

```json
{
  "theme": "dark",
  "shortcut": "Ctrl+Alt+x",
  "autostart": true,
  "check_updates": true,
  "window_width": 400,
  "window_height": 600
}
```

| Campo | Tipo | Padrao | Descricao |
|---|---|---|---|
| theme | string | `"dark"` | `"dark"` ou `"light"` |
| shortcut | string | `"Ctrl+Alt+x"` | Atalho global para abrir/fechar |
| autostart | boolean | `true` | Iniciar com o Windows |
| check_updates | boolean | `true` | Verificar atualizacoes no GitHub |
| window_width | number | `400` | Largura da popup em pixels |
| window_height | number | `600` | Altura da popup em pixels |

---

## 5. System Tray

### 5.1 Icone
- Icone proprio do Recall na bandeja do sistema
- Tooltip: "Recall"

### 5.2 Menu de Contexto (botao direito)
```
Recall
─────────────
Abrir
Nova Nota
Novo Lembrete
─────────────
Configuracoes
─────────────
Sair
```

### 5.3 Comportamento da Janela
- **Duplo clique no icone** = toggle abre/fecha a popup
- **Botao direito** = menu de contexto
- **Atalho global** (Ctrl+Alt+x) = toggle abre/fecha
- **Popup** posicionada no canto inferior direito, proximo ao tray
- **Tamanho padrao:** 400x600px, configuravel
- **Fechar (X)** = minimiza para tray (nao fecha o app)
- **Esc** = fecha a popup
- **Clique fora da popup** = fecha a popup

---

## 6. Scheduler de Lembretes

### 6.1 Ciclo de Verificacao
- Timer em background roda a cada **30 segundos**
- Para cada lembrete com `status == "pending"`:
  - Se `trigger_at` <= agora: dispara notificacao
  - Se `repeat` definido: atualiza `trigger_at` para o proximo ciclo
  - Se `repeat` nulo: marca `status` como `"fired"`

### 6.2 Lembretes Perdidos
- Ao iniciar o app, verifica lembretes pendentes com `trigger_at` no passado
- Dispara todos imediatamente (o usuario nao perde lembretes por ter o PC desligado)

### 6.3 Notificacoes
- Usa `tauri-plugin-notification` para notificacoes nativas do Windows
- Titulo: titulo do lembrete
- Corpo: descricao do lembrete (se houver)
- Ao clicar na notificacao: abre o Recall e destaca o lembrete

### 6.4 Recorrencia
| repeat | Proximo trigger_at |
|---|---|
| `"daily"` | trigger_at + 1 dia |
| `"weekly"` | trigger_at + 7 dias |
| `"monthly"` | trigger_at + 1 mes |

---

## 7. Frontend (UI)

### 7.1 Navegacao
Navegacao por abas na parte superior:
- **Notas** (icone de caderno)
- **Lembretes** (icone de sino)
- **Config** (icone de engrenagem)

### 7.2 Tela: Notas

**Layout:**
```
┌─────────────────────────────────┐
│ [Busca.................] [+]    │
│ [Categoria: Todas ▼] [Tag: ▼]  │
├─────────────────────────────────┤
│ ★ Nota fixada                   │
│   Trabalho · #reuniao · 27/05   │
├─────────────────────────────────┤
│   Outra nota                    │
│   Pessoal · 26/05               │
├─────────────────────────────────┤
│   Mais uma nota                 │
│   Estudos · #urgente · 25/05    │
└─────────────────────────────────┘
```

**Funcionalidades:**
- Busca por titulo e conteudo (filtro em tempo real)
- Filtro por categoria (dropdown)
- Filtro por tag (dropdown)
- Notas fixadas aparecem no topo
- Clique na nota abre o editor
- Botao "+" cria nova nota

**Editor de nota:**
```
┌─────────────────────────────────┐
│ ← Voltar         [Salvar] [🗑] │
├─────────────────────────────────┤
│ Titulo: [.....................] │
│                                 │
│ Categoria: [Trabalho     ▼]    │
│ Tags: [urgente] [reuniao] [+]  │
│                                 │
│ ┌─────────────────────────────┐ │
│ │                             │ │
│ │   Corpo da nota...          │ │
│ │                             │ │
│ │                             │ │
│ └─────────────────────────────┘ │
│                                 │
│ Criada: 27/05/2026 14:00       │
│ Editada: 27/05/2026 14:30      │
└─────────────────────────────────┘
```

### 7.3 Tela: Lembretes

**Layout:**
```
┌─────────────────────────────────┐
│ [+] Novo Lembrete               │
├─────────────────────────────────┤
│ ⏰ Pagar conta de luz           │
│   28/05 10:00 · Pendente        │
│   Vence dia 30                  │
├─────────────────────────────────┤
│ 🔔 Daily standup                │
│   Diario 09:00 · Recorrente     │
├─────────────────────────────────┤
│ ✓ Reuniao projeto               │
│   27/05 14:00 · Disparado       │
└─────────────────────────────────┘
```

**Formulario de novo lembrete:**
```
┌─────────────────────────────────┐
│ ← Voltar         [Salvar]      │
├─────────────────────────────────┤
│ Titulo: [.....................] │
│ Descricao: [..................] │
│                                 │
│ Tipo: ○ Data/hora exata         │
│       ○ Recorrencia             │
│       ○ Timer relativo          │
│                                 │
│ [Se data/hora:]                 │
│ Data: [28/05/2026] Hora: [10:00]│
│                                 │
│ [Se recorrencia:]               │
│ Repetir: [Diario ▼]             │
│                                 │
│ [Se timer:]                     │
│ Em: [30] minutos                │
│                                 │
│ Vincular nota: [Nenhuma    ▼]   │
└─────────────────────────────────┘
```

### 7.4 Tela: Configuracoes

```
┌─────────────────────────────────┐
│ Configuracoes                   │
├─────────────────────────────────┤
│                                 │
│ Tema:          ○ Dark ● Light   │
│                                 │
│ Atalho global: [Ctrl+Alt+x ]    │
│                                 │
│ Iniciar com    [✓]              │
│ o Windows:                      │
│                                 │
│ Verificar      [✓]              │
│ atualizacoes:                   │
│                                 │
│ Versao: 1.0.0                   │
│ [Verificar atualizacoes]        │
│                                 │
└─────────────────────────────────┘
```

---

## 8. Temas

### 8.1 Dark (padrao)
```css
--bg-primary: #1a1a2e;
--bg-secondary: #16213e;
--bg-card: #0f3460;
--text-primary: #e0e0e0;
--text-secondary: #a0a0a0;
--accent: #e94560;
--accent-hover: #ff6b81;
--border: #2a2a4a;
```

### 8.2 Light
```css
--bg-primary: #f5f5f5;
--bg-secondary: #ffffff;
--bg-card: #ffffff;
--text-primary: #1a1a1a;
--text-secondary: #666666;
--accent: #e94560;
--accent-hover: #c0392b;
--border: #e0e0e0;
```

Tema alternado via classe no `<body>` (`class="dark"` ou `class="light"`). CSS usa variaveis que mudam com a classe.

---

## 9. Auto-Start

- Plugin `tauri-plugin-autostart` registra o app no Registro do Windows (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`)
- Configuravel via toggle na tela de Configuracoes
- Padrao: **habilitado**

---

## 10. Auto-Update

- Plugin `tauri-plugin-updater` verifica releases no GitHub
- Endpoint configurado no `tauri.conf.json` apontando para o repo GitHub
- Fluxo:
  1. Ao iniciar (se `check_updates == true`), verifica se ha nova versao
  2. Se houver: mostra notificacao "Atualizacao disponivel"
  3. Usuario clica "Atualizar" ou a atualizacao e silenciosa (configuravel)
  4. Download + instalacao automatica
  5. Reinicia o app
- Para gerar atualizacoes: criar release no GitHub com os artefatos gerados pelo Tauri build

---

## 11. Instalador

- Tauri gera instalador `.msi` via WiX
- Wizard padrao: Next > Instalar > Concluir
- Instala em `C:\Users\{user}\AppData\Local\Recall`
- Cria atalho na Area de Trabalho (opcional) e no Menu Iniciar
- Desinstalador incluido automaticamente

---

## 12. Estrutura de Pastas no Disco

**Em producao** (app instalado):
```
%APPDATA%/Recall/
├── data/
│   ├── notes/
│   │   ├── a1b2c3d4.json
│   │   └── ...
│   ├── reminders/
│   │   ├── e5f6g7h8.json
│   │   └── ...
│   └── config.json
└── logs/              # Logs de erro (opcional)
```

**Em desenvolvimento** (`cargo tauri dev`):
```
src-tauri/data/        # Mesma estrutura, dentro do projeto
```

O Tauri permite configurar paths diferentes via `tauri::api::path`. Em dev, os dados ficam no projeto para facil acesso. Em producao, usamos `%APPDATA%` como padrao Windows.

---

## 13. Comandos IPC (Rust <-> Frontend)

| Comando | Parametros | Retorno | Descricao |
|---|---|---|---|
| `get_notes` | `{ filter? }` | `Note[]` | Lista notas com filtros opcionais |
| `get_note` | `{ id }` | `Note` | Busca nota por ID |
| `create_note` | `{ title, content?, category?, tags? }` | `Note` | Cria nova nota |
| `update_note` | `{ id, ...fields }` | `Note` | Atualiza nota existente |
| `delete_note` | `{ id }` | `void` | Remove nota |
| `get_reminders` | `{ status? }` | `Reminder[]` | Lista lembretes |
| `create_reminder` | `{ ...campos }` | `Reminder` | Cria lembrete |
| `update_reminder` | `{ id, ...fields }` | `Reminder` | Atualiza lembrete |
| `delete_reminder` | `{ id }` | `void` | Remove lembrete |
| `dismiss_reminder` | `{ id }` | `void` | Marca lembrete como dispensado |
| `get_config` | - | `Config` | Retorna configuracoes |
| `update_config` | `{ ...fields }` | `Config` | Atualiza configuracoes |
| `get_categories` | - | `string[]` | Lista categorias usadas |
| `get_tags` | - | `string[]` | Lista tags usadas |

---

## 14. Testes

### 14.1 Testes Rust (backend)
- **Unit tests:** Funcoes de storage (CRUD JSON), scheduler (calculo de triggers), parsing de configuracao
- **Integration tests:** Comandos IPC completos (criar nota -> listar -> editar -> deletar)
- **Framework:** `#[cfg(test)]` nativo do Rust

### 14.2 Testes Frontend
- **Unit tests:** Funcoes utilitarias (formatacao de data, filtros)
- **E2E tests:** Fluxos principais (criar nota, criar lembrete, mudar tema)
- **Framework:** Playwright ou WebDriver (via Tauri)

### 14.3 Cenarios criticos para testar
- Corrupcao de JSON (arquivo malformado)
- Lembretes perdidos (PC desligado, depois liga)
- Concorrencia (dois writes simultaneos)
- Atalho global em conflito com outro app
- Migração de dados entre versoes

---

## 15. Fluxo de Build e Distribuicao

### Desenvolvimento
```bash
cargo tauri dev    # Dev com hot-reload
```

### Producao
```bash
cargo tauri build  # Gera .msi + .exe
```

### Release
1. Atualizar versao em `Cargo.toml` e `tauri.conf.json`
2. Push tag `v1.0.0` no GitHub
3. GitHub Actions gera os artefatos (.msi, .exe, update JSON)
4. Criar release no GitHub com os artefatos
5. App verifica atualizacao sozinho

---

## 16. Stack Resumido

```
Tauri 2.x
  ├── Rust (backend)
  │   ├── tauri-plugin-autostart
  │   ├── tauri-plugin-updater
  │   ├── tauri-plugin-notification
  │   └── tauri-plugin-shell
  ├── HTML/CSS/JS (frontend, vanilla)
  ├── WebView2 (renderizacao, nativo Windows 10/11)
  └── JSON (armazenamento local)
```

**Zero dependencias JavaScript.** Apenas Tauri + Rust.

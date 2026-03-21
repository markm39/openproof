# openproof

`openproof` is a Rust-first, local-first formal math agent for Lean 4. The native shell, native store, native dashboard server, and verified artifacts stay visible while Lean owns final verification.

## Crates

- `crates/openproof-cli` -- main CLI binary with TUI event loop and command system
- `crates/openproof-tui` -- terminal UI rendering (ratatui)
- `crates/openproof-core` -- application state machine and event handling
- `crates/openproof-store` -- SQLite persistence (sessions, corpus, sync queue)
- `crates/openproof-protocol` -- shared types (serde-serializable, no I/O)
- `crates/openproof-model` -- ChatGPT Codex API integration and authentication
- `crates/openproof-lean` -- Lean toolchain interaction and verification
- `crates/openproof-dashboard` -- HTTP dashboard server (Axum)
- `crates/openproof-cloud` -- HTTP client for the remote corpus API
- `crates/openproof-corpus` -- corpus orchestration (lake ingestion, search, sync)
- `crates/openproof-corpus-server` -- shared corpus HTTP server with quarantine and reverification

Run the product with:

```bash
./bin/openproof
```

## Cloud Boundary

The public `openproof` repo is client-side and local-first:

- TUI and dashboard
- local verified corpus and cache
- local Lean verification
- public shared-corpus wire types and HTTP client
- local development reference server

Prelaunch policy:

- `local` is the default and intended mode
- `community` and `private` remain visible, but are dev-gated
- remote corpus behavior is only enabled when both `OPENPROOF_ENABLE_REMOTE_CORPUS=1` and `OPENPROOF_CORPUS_URL` are set
- corpus auth is kept separate from ChatGPT/Codex model auth

The production shared corpus moat is not meant to live here. The canonical verified corpus service, reverification pipeline, ranking logic, secrets, and infrastructure belong in a private `openproof-cloud` repo.

## Corpus modes

`openproof` supports three corpus modes in the public client:

- `local`: fully local only. Search, verification, and corpus growth stay on disk in `~/.openproof`.
- `community`: opt into the hosted shared corpus. The product model is contribution-for-access: users contribute structured verified artifacts in exchange for shared-corpus retrieval.
- `private`: use a tenant-scoped hosted corpus. Private uploads stay private, and community overlay is optional for retrieval.

Important boundaries:

- the public repo does not ship the production corpus service
- raw chat logs are not the default contribution unit
- only structured verified artifacts are part of the shared-corpus contribution flow
- traces and failed attempts remain local/internal telemetry, not shared retrieval data
- the near-term self-host story is local-first open source, not a full self-hosted hosted-corpus stack

## Setup

```bash
cd lean && lake update
cd ..
cargo build --workspace
```

Lean verification depends on a populated local Lean 4 + mathlib environment. The pinned project lives in [`lean`](./lean).

## Usage

```bash
openproof
openproof health
openproof login
openproof dashboard --open
openproof recluster-corpus
```

Corpus server (separate binary):

```bash
cargo run -p openproof-corpus-server -- --port 4832
```

## Interactive commands

- `/help`
- `/status`
- `/login`
- `/logout`
- `/mode benchmark|research`
- `/model [name]`
- `/plan`
- `/compact`
- `/proof`
- `/share local|community|private`
- `/share overlay [on|off]`
- `/instructions`
- `/memory`
- `/remember <text>`
- `/remember global <text>`
- `/paper [tex]`
- `/new <title>`
- `/resume <session-id>`
- `/theorem <label> :: <statement>`
- `/lemma <label> :: <statement>`
- `/verify`
- `/agents`
- `/tasks`
- `/branches`
- `/focus <branch-id|node-id|clear>`
- `/agent spawn <role> <task>`
- `/export paper|tex|lean|all`
- `/corpus status`
- `/corpus ingest`
- `/corpus recluster`
- `/sync status|enable|disable|drain`
- `/autonomous start|stop|status|step`
- `/sessions`
- `/dashboard`
- `/feedback`

Keybindings:

- `Ctrl+C` interrupts the active foreground model turn
- `Ctrl+D` exits the TUI
- `Shift+Tab` cycles shell mode between `default` and `plan`
- `Esc` clears the current input

Any non-command line is treated as a chat turn. The proof graph is preserved across the chat and is stored independently from the transcript.

`openproof` also loads inherited `AGENTS.md` files and persistent memory:

- global memory: `~/.openproof/memory/global.md`
- workspace memory: `~/.openproof/memory/workspaces/<workspace-hash>.md`

## OAuth notes

`openproof` first tries to reuse an existing Codex CLI ChatGPT login from `~/.codex/auth.json` or the macOS Codex credential store. If no reusable Codex session is present, it falls back to its own browser-based ChatGPT OAuth flow.

The implementation follows the documented Codex login shape:

- Codex caches ChatGPT login details locally and reuses them across runs
- browser login is the default sign-in flow when no cached session is available
- the CLI uses a localhost callback to return the OAuth result
- active ChatGPT sessions are refreshed and cached locally

Sources:

- https://developers.openai.com/codex/auth/
- https://developers.openai.com/codex/cli/reference/#codex-login

Headless device-code auth is not implemented yet. Right now `openproof` assumes a local browser-capable environment.

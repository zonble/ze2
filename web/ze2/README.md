# ze2 Web POC

This directory contains the first browser-only proof of concept for ze2.

The POC uses:

- `ze2-web` compiled to `wasm32-unknown-unknown`
- xterm.js as the browser terminal host
- a small static Node server for local testing

## Build

From the repository root:

```powershell
cargo build -p ze2-web --target wasm32-unknown-unknown --release
Copy-Item target\wasm32-unknown-unknown\release\ze2_web.wasm web\ze2\ze2_web.wasm
```

## Run Locally

```powershell
cd web\ze2
node server.mjs
```

Then open:

```text
http://127.0.0.1:8080/
```

## Current Scope

This is not the full native ze2 editor yet. It proves the browser-only architecture by running ze2's TUI framework in WASM and rendering its VT output through xterm.js.

Currently supported:

- WASM initialization
- xterm.js rendering
- resize events
- basic text input
- basic editing keys such as Backspace and arrows
- browser open/save flow for a single text buffer
- browser IndexedDB autosave/restore for editor buffers

Not yet supported:

- Native ze2 `main.rs`
- Multi-document editor state
- Directory browsing
- OPFS workspace persistence
- Full VT input parsing
- Clipboard integration
- GitHub Pages build automation

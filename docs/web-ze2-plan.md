# Web ze2 Plan

## Goal

Build a browser-hosted version of ze2 that can eventually be published as a static web app, including GitHub Pages. The long-term target is a pure frontend deployment:

```text
GitHub Pages
  index.html
  JavaScript
  xterm.js
  ze2.wasm
  browser file/storage APIs
```

This is different from running the native ze2 binary behind a terminal server. A server-backed proof of concept is useful for quick validation, but it cannot be deployed on GitHub Pages by itself.

## Deployment Constraints

GitHub Pages can only serve static assets. It cannot:

- Run a native ze2 process.
- Host a PTY.
- Run a WebSocket backend.
- Access the user's host filesystem directly.

Therefore, a GitHub Pages-compatible version must run entirely in the browser. The executable logic has to be compiled to WebAssembly, and all host services must be provided through browser APIs.

## Candidate Architectures

### 1. Native ze2 + PTY server + xterm.js

```text
browser xterm.js <-> WebSocket <-> server PTY <-> native ze2 <-> host filesystem
```

This is the fastest way to see ze2 working inside a browser terminal. It preserves the current terminal application model: ze2 writes VT escape sequences to stdout and reads keyboard/mouse input from stdin.

Benefits:

- Minimal changes to ze2.
- Existing file I/O keeps working through the host filesystem.
- xterm.js can render the terminal output directly.

Limitations:

- Requires a backend server.
- Cannot be hosted only on GitHub Pages.
- Needs workspace isolation and filesystem permission controls.

This path is useful as an exploratory demo, but it is not the target architecture for a static web release.

### 2. Browser WASM + xterm.js

```text
browser xterm.js <-> JavaScript bridge <-> ze2.wasm <-> browser storage APIs
```

This keeps the terminal rendering model but moves ze2 into WebAssembly. xterm.js remains responsible for displaying VT output and collecting keyboard/mouse input.

Benefits:

- Compatible with static hosting if all assets are generated at build time.
- Reuses ze2's existing VT-oriented rendering path.
- Avoids writing a custom canvas/DOM renderer in the first web version.

Costs:

- ze2 and stdext need wasm platform support.
- The blocking stdin/stdout application loop needs a browser-compatible bridge.
- File I/O must be adapted to browser APIs.
- Clipboard, window title, terminal sizing, and other host interactions need browser-specific handling.

This is the recommended first GitHub Pages-compatible proof of concept.

### 3. Browser WASM + custom renderer

```text
browser renderer <-> JavaScript bridge <-> ze2.wasm framebuffer/cell grid
```

This avoids terminal emulation and renders ze2's UI directly using DOM, canvas, or WebGL.

Benefits:

- More native web experience.
- Better long-term control over layout, font rendering, pointer behavior, accessibility, and styling.

Costs:

- Larger implementation effort.
- Requires a stable API for exposing framebuffer or cell-grid data from Rust to JavaScript.
- More web-specific rendering work before the editor becomes usable.

This is a better long-term direction only after the WASM core and browser I/O model are proven.

## Current Codebase Observations

The current native entry point is `crates/ze2/src/bin/ze2/main.rs`. It is built around a terminal loop:

- Initialize platform services.
- Parse CLI arguments.
- Read stdin and files.
- Switch the terminal into raw mode.
- Read input from stdin.
- Parse VT/input events.
- Draw the TUI.
- Render VT output.
- Write to stdout.

The shared UI code already has a useful separation: `Tui` builds an immediate-mode UI tree and renders terminal output. That makes xterm.js a practical first browser renderer.

The current workspace does not compile for `wasm32-unknown-unknown` yet. A first check fails in `stdext` because the arena allocator expects platform virtual memory functions:

- `virtual_reserve`
- `virtual_commit`
- `virtual_release`

Those are currently implemented for Unix and Windows, but not for wasm.

## File I/O Strategy

Native filesystem access is not available in a pure browser app. The web version should define a browser file abstraction instead of trying to preserve unrestricted `std::fs` behavior.

Initial options:

- File upload/open: read a user-selected file into an untitled or named buffer.
- Download/save: write the current buffer as a downloaded file.
- File System Access API: where available, save back to a selected file handle.
- OPFS: use Origin Private File System for autosave, scratch files, recent sessions, and internal workspace state.
- IndexedDB: possible backing store for metadata or compatibility fallback.

Recommended POC scope:

- Start with one untitled buffer.
- Add "open text file" through browser file selection.
- Add "save/download" through a generated Blob.
- Defer directory browsing and full workspace semantics.

Later scope:

- Persist recent documents in OPFS.
- Support save-back through `FileSystemFileHandle` when the browser supports it.
- Add a virtual filesystem layer if ze2 needs path-oriented APIs internally.

## Browser Host Services

The web host must replace terminal/OS services currently provided by `ze2::sys`.

Required services for the first browser POC:

- Terminal input from xterm.js.
- Terminal output to xterm.js.
- Resize events from the browser terminal viewport.
- Timer or animation wakeups for non-blocking rendering.
- Panic/error reporting to the browser console and terminal.

Services that can be reduced or deferred:

- Clipboard sync.
- Terminal title changes.
- Directory picker.
- CLI argument parsing.
- Reading redirected stdin.
- Full mouse support.
- Localized terminal capability queries.

## WASM Porting Work

### stdext platform support

Add wasm-compatible implementations for the arena platform hooks. Since wasm linear memory does not have the same reserve/commit model as native virtual memory, a simple POC implementation can allocate a backing buffer up front and treat commit as a bounds check/no-op.

This should be isolated behind `cfg(target_arch = "wasm32")`.

### ze2 platform support

Add a wasm/browser platform module for `ze2::sys`, or introduce a host trait that can be implemented by the native terminal host and the browser host.

The browser host should not call native terminal APIs. It should receive input from JavaScript and return output/events to JavaScript.

### Event loop

The current main loop blocks on `sys::read_stdin()`. Browser code is event-driven. The web version should expose functions such as:

- initialize editor state
- resize terminal
- push terminal input bytes
- render pending output
- request save/open actions through JavaScript

A Web Worker can be considered later if rendering or parsing becomes expensive.

## Recommended POC

Build the first static-web POC around `wasm32-unknown-unknown`, JavaScript glue, and xterm.js.

POC scope:

- Compile enough of `stdext` and `ze2` to initialize editor state in WASM.
- Create a browser-specific entry point instead of using the native `main.rs`.
- Render ze2 VT output into xterm.js.
- Send xterm.js keyboard input into the WASM editor.
- Start with an untitled buffer.
- Implement simple open/download file flow in JavaScript.
- Serve the result as static files.

Out of scope for the first POC:

- Native PTY backend.
- GitHub Pages deployment automation.
- Full filesystem emulation.
- Multi-file workspace directory browsing.
- Perfect clipboard integration.
- Custom canvas renderer.

## Current POC Status

An initial implementation exists under `crates/ze2-web` and `web/ze2`.

This version compiles ze2's shared TUI framework to `wasm32-unknown-unknown` and renders into xterm.js from a static browser page. It is intentionally smaller than the native editor: it uses a single demo text buffer and browser open/download actions instead of the native multi-document/file-system workflow.

The first platform shims are also present:

- `crates/stdext/src/sys/wasm.rs` provides arena memory hooks for wasm.
- `crates/ze2/src/sys/wasm.rs` provides minimal browser-compatible host stubs.

The POC is meant to validate the static web architecture before porting the full native ze2 editor state and command modules.

## Milestones

1. Add wasm platform shims for `stdext`.
2. Create a web-specific ze2 entry point.
3. Build a JavaScript/xterm.js host page.
4. Drive resize and keyboard input from xterm.js into WASM.
5. Render terminal output from WASM into xterm.js.
6. Add minimal browser file open/download support.
7. Produce static build artifacts suitable for GitHub Pages.

## Open Questions

- Should the first browser target be `wasm32-unknown-unknown` with custom JS glue, or `wasm32-wasip1` with a WASI polyfill?
- Should the browser editor preserve VT output as the primary rendering contract, or expose a framebuffer/cell-grid API early?
- How much of the native CLI behavior should exist in the browser version?
- What is the minimum acceptable file model: single document, multiple selected files, or OPFS-backed workspace?
- Should the browser version live in this workspace as a new crate, or as a separate web package that depends on `ze2`?

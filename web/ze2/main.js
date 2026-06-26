const term = new Terminal({
  convertEol: true,
  cursorBlink: true,
  fontFamily: fontFamilyForChoice(readStoredSettings().fontFamily),
  fontSize: 21,
});
const fitAddon = new FitAddon.FitAddon();
term.loadAddon(fitAddon);
const terminalElement = document.getElementById("terminal");
term.open(terminalElement);
enableTerminalModes();
terminalElement.addEventListener("contextmenu", (event) => {
  event.preventDefault();
});

const wasmResponse = await fetch("./ze2_web.wasm?real-editor=1", { cache: "no-store" });
let wasm;
try {
  wasm = await WebAssembly.instantiateStreaming(Promise.resolve(wasmResponse.clone()), {});
} catch {
  wasm = await WebAssembly.instantiate(await wasmResponse.arrayBuffer(), {});
}
const api = wasm.instance.exports;
const encoder = new TextEncoder();
const decoder = new TextDecoder();

function writeBytes(bytes) {
  const ptr = api.ze2_web_alloc(bytes.length);
  new Uint8Array(api.memory.buffer, ptr, bytes.length).set(bytes);
  return { ptr, len: bytes.length };
}

function readBytes(ptr, len) {
  return new Uint8Array(api.memory.buffer, ptr, len).slice();
}

function flush() {
  const ptr = api.ze2_web_output_ptr();
  const len = api.ze2_web_output_len();
  if (ptr && len) {
    term.write(readBytes(ptr, len));
  }
}

function enableTerminalModes() {
  term.write("\x1b[?1002;1006;2004h");
}

function repaint() {
  api.ze2_web_redraw();
  term.reset();
  enableTerminalModes();
  flush();
}

let escFlushTimer = 0;

const HOST_ACTION_OPEN = 1;
const HOST_ACTION_SAVE = 2;
const HOST_ACTION_CLIPBOARD_READ = 3;
const HOST_ACTION_CLIPBOARD_WRITE = 4;
const SETTINGS_KEY = "ze2-web-settings";

function sendInput(data) {
  const bytes = encoder.encode(data);
  const input = writeBytes(bytes);
  api.ze2_web_input(input.ptr, input.len);
  api.ze2_web_dealloc(input.ptr, input.len);

  if (escFlushTimer) {
    clearTimeout(escFlushTimer);
    escFlushTimer = 0;
  }

  if (data.endsWith("\x1b")) {
    escFlushTimer = setTimeout(() => {
      api.ze2_web_flush_input();
      flush();
    }, 120);
  }

  flush();
  void handleHostAction();
  persistEditorSettings();
}

function resize() {
  fitAddon.fit();
  api.ze2_web_resize(term.cols, term.rows);
  term.reset();
  enableTerminalModes();
  flush();
  void handleHostAction();

  requestAnimationFrame(() => {
    fitAddon.fit();
    api.ze2_web_resize(term.cols, term.rows);
    repaint();
    void handleHostAction();
  });
}

let resizeFrame = 0;

function scheduleResize() {
  if (resizeFrame) {
    cancelAnimationFrame(resizeFrame);
  }

  resizeFrame = requestAnimationFrame(() => {
    resizeFrame = 0;
    resize();
  });
}

fitAddon.fit();
if (!api.ze2_web_init(term.cols, term.rows)) {
  term.writeln("failed to initialize ze2_web.wasm");
} else {
  applyStoredSettings();
  term.clear();
  flush();
}

term.onData((data) => {
  sendInput(data);
});

window.addEventListener("keydown", (event) => {
  if (!event.ctrlKey || event.altKey || event.metaKey || event.shiftKey) {
    return;
  }

  const key = event.key.toLowerCase();
  if (key === "o") {
    event.preventDefault();
    sendInput("\x0f");
  } else if (key === "s") {
    event.preventDefault();
    sendInput("\x13");
  } else if (key === "c") {
    event.preventDefault();
    sendInput("\x03");
  } else if (key === "x") {
    event.preventDefault();
    sendInput("\x18");
  } else if (key === "v") {
    event.preventDefault();
    sendInput("\x16");
  }
});

window.addEventListener("resize", scheduleResize);
new ResizeObserver(scheduleResize).observe(terminalElement);

document.getElementById("open").addEventListener("click", async () => {
  await openBrowserFile();
});

document.getElementById("save").addEventListener("click", () => {
  saveBrowserFile();
});

for (const input of document.querySelectorAll('input[name="font"]')) {
  input.addEventListener("change", () => {
    if (input.checked) {
      applyFontChoice(input.value);
      persistEditorSettings();
    }
  });
}

async function handleHostAction() {
  switch (api.ze2_web_take_host_action()) {
    case HOST_ACTION_OPEN:
      await openBrowserFile();
      break;
    case HOST_ACTION_SAVE:
      saveBrowserFile();
      break;
    case HOST_ACTION_CLIPBOARD_READ:
      await pasteFromSystemClipboard();
      break;
    case HOST_ACTION_CLIPBOARD_WRITE:
      await copyToSystemClipboard();
      break;
  }
  persistEditorSettings();
}

function applyStoredSettings() {
  const settings = readStoredSettings();
  api.ze2_web_apply_settings(
    settings.wordWrap ? 1 : 0,
    settings.wordWrapColumn,
    settings.ruler ? 1 : 0,
    settings.centerText ? 1 : 0,
    settings.highlightCurrentChar ? 1 : 0,
    settings.editorColor,
  );
  applyFontChoice(settings.fontFamily);
}

function readStoredSettings() {
  const defaults = {
    wordWrap: false,
    wordWrapColumn: 0,
    ruler: false,
    centerText: false,
    highlightCurrentChar: false,
    editorColor: 0,
    fontFamily: "standard",
  };

  try {
    return { ...defaults, ...JSON.parse(localStorage.getItem(SETTINGS_KEY) || "{}") };
  } catch {
    return defaults;
  }
}

function persistEditorSettings() {
  const settings = {
    wordWrap: api.ze2_web_setting_word_wrap() !== 0,
    wordWrapColumn: api.ze2_web_setting_word_wrap_column(),
    ruler: api.ze2_web_setting_ruler() !== 0,
    centerText: api.ze2_web_setting_center_text() !== 0,
    highlightCurrentChar: api.ze2_web_setting_highlight_current_char() !== 0,
    editorColor: api.ze2_web_setting_editor_color(),
    fontFamily: document.querySelector('input[name="font"]:checked')?.value || "standard",
  };
  localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
}

function fontFamilyForChoice(choice) {
  if (choice === "kai") {
    return '"DFKai-SB", "BiauKai", KaiTi, "TW-Kai", serif';
  }
  return 'Consolas, "Cascadia Mono", "SFMono-Regular", monospace';
}

function applyFontChoice(choice) {
  const normalized = choice === "kai" ? "kai" : "standard";
  term.options.fontFamily = fontFamilyForChoice(normalized);
  document.getElementById("font-standard").checked = normalized === "standard";
  document.getElementById("font-kai").checked = normalized === "kai";
  scheduleResize();
}

async function copyToSystemClipboard() {
  const ptr = api.ze2_web_clipboard_ptr();
  const len = api.ze2_web_clipboard_len();
  const text = decoder.decode(readBytes(ptr, len));
  await navigator.clipboard.writeText(text);
  api.ze2_web_mark_clipboard_synced();
}

async function pasteFromSystemClipboard() {
  const text = await navigator.clipboard.readText();
  const bytes = encoder.encode(text);
  const paste = writeBytes(bytes);
  api.ze2_web_paste(paste.ptr, paste.len);
  api.ze2_web_dealloc(paste.ptr, paste.len);
  flush();
}

async function openBrowserFile() {
  const [file] = await showOpenFilePicker({
    types: [{ description: "Text", accept: { "text/*": [".txt", ".md", ".rs"] } }],
  });
  const text = await (await file.getFile()).text();
  const bytes = encoder.encode(text);
  const doc = writeBytes(bytes);
  api.ze2_web_set_document(doc.ptr, doc.len);
  api.ze2_web_dealloc(doc.ptr, doc.len);
  term.clear();
  flush();
}

function saveBrowserFile() {
  const ptr = api.ze2_web_document_ptr();
  const len = api.ze2_web_document_len();
  const text = decoder.decode(readBytes(ptr, len));
  const blob = new Blob([text], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = "ze2-web.txt";
  anchor.click();
  URL.revokeObjectURL(url);
}

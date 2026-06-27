const term = new Terminal({
  allowProposedApi: true,
  convertEol: true,
  cursorBlink: true,
  fontFamily: fontFamilyForChoice(readStoredSettings().fontFamily),
  fontSize: 21,
});
const fitAddon = new FitAddon.FitAddon();
term.loadAddon(fitAddon);
enableUnicode11();
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

function enableUnicode11() {
  if (!window.Unicode11Addon?.Unicode11Addon || !term.unicode) {
    return;
  }

  term.loadAddon(new window.Unicode11Addon.Unicode11Addon());
  term.unicode.activeVersion = "11";
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
const AUTOSAVE_DB_NAME = "ze2-web";
const AUTOSAVE_DB_VERSION = 1;
const AUTOSAVE_STORE_NAME = "buffers";
const AUTOSAVE_META_ID = "__meta__";
const DEFAULT_BUFFER_NAME = "Untitled-1.txt";

let autosaveTimer = 0;
let lastAutosavedId = "";
let lastAutosavedText = "";

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
  scheduleAutosave();
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
  await restoreAutosavedBuffer();
  applyStoredSettings();
  term.clear();
  flush();
  scheduleAutosave();
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
  scheduleAutosave();
}

async function openBrowserFile() {
  const [file] = await showOpenFilePicker({
    types: [{ description: "Text", accept: { "text/*": [".txt", ".md", ".rs"] } }],
  });
  const text = await (await file.getFile()).text();
  setDocumentText(text, file.name || DEFAULT_BUFFER_NAME);
  term.clear();
  flush();
  await autosaveNow().catch(reportAutosaveError);
}

function saveBrowserFile() {
  const text = readCurrentDocumentText();
  const blob = new Blob([text], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = readActiveBufferName();
  anchor.click();
  URL.revokeObjectURL(url);
  void autosaveNow().catch(reportAutosaveError);
}

async function restoreAutosavedBuffer() {
  let saved;
  try {
    const meta = await getAutosavedBuffer(AUTOSAVE_META_ID);
    saved = await getAutosavedBuffer(meta?.lastActiveBufferId || bufferIdForName(DEFAULT_BUFFER_NAME));
  } catch (error) {
    reportAutosaveError(error);
  }

  if (!saved) {
    lastAutosavedId = bufferIdForName(readActiveBufferName());
    lastAutosavedText = readCurrentDocumentText();
    return;
  }

  const text = saved.text || "";
  setDocumentText(text, saved.name || saved.id || DEFAULT_BUFFER_NAME);
  lastAutosavedId = bufferIdForName(readActiveBufferName());
  lastAutosavedText = text;
}

function scheduleAutosave() {
  if (autosaveTimer) {
    clearTimeout(autosaveTimer);
  }

  autosaveTimer = setTimeout(() => {
    autosaveTimer = 0;
    void autosaveNow().catch(reportAutosaveError);
  }, 400);
}

window.addEventListener("pagehide", () => {
  void autosaveNow().catch(reportAutosaveError);
});

document.addEventListener("visibilitychange", () => {
  if (document.visibilityState === "hidden") {
    void autosaveNow().catch(reportAutosaveError);
  }
});

async function autosaveNow() {
  const text = readCurrentDocumentText();
  const name = readActiveBufferName();
  const id = bufferIdForName(name);
  if (id === lastAutosavedId && text === lastAutosavedText) {
    return;
  }

  await putAutosavedBuffer({
    id,
    name,
    text,
    updatedAt: Date.now(),
  });
  await putAutosavedBuffer({
    id: AUTOSAVE_META_ID,
    lastActiveBufferId: id,
    updatedAt: Date.now(),
  });
  lastAutosavedId = id;
  lastAutosavedText = text;
}

function readCurrentDocumentText() {
  const ptr = api.ze2_web_document_ptr();
  const len = api.ze2_web_document_len();
  return decoder.decode(readBytes(ptr, len));
}

function readActiveBufferName() {
  const ptr = api.ze2_web_document_name_ptr?.();
  const len = api.ze2_web_document_name_len?.() || 0;
  if (!ptr || !len) {
    return DEFAULT_BUFFER_NAME;
  }
  return decoder.decode(readBytes(ptr, len)) || DEFAULT_BUFFER_NAME;
}

function setDocumentText(text, name) {
  const textBytes = encoder.encode(text);
  const nameBytes = encoder.encode(name);
  const doc = writeBytes(textBytes);
  const docName = writeBytes(nameBytes);
  try {
    if (api.ze2_web_set_document_with_name) {
      api.ze2_web_set_document_with_name(doc.ptr, doc.len, docName.ptr, docName.len);
    } else {
      api.ze2_web_set_document(doc.ptr, doc.len);
    }
  } finally {
    api.ze2_web_dealloc(doc.ptr, doc.len);
    api.ze2_web_dealloc(docName.ptr, docName.len);
  }
}

function bufferIdForName(name) {
  return name || DEFAULT_BUFFER_NAME;
}

function openAutosaveDb() {
  return new Promise((resolve, reject) => {
    if (!window.indexedDB) {
      reject(new Error("IndexedDB is not available"));
      return;
    }

    const request = indexedDB.open(AUTOSAVE_DB_NAME, AUTOSAVE_DB_VERSION);
    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains(AUTOSAVE_STORE_NAME)) {
        db.createObjectStore(AUTOSAVE_STORE_NAME, { keyPath: "id" });
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

async function getAutosavedBuffer(id) {
  const db = await openAutosaveDb();
  try {
    return await new Promise((resolve, reject) => {
      const tx = db.transaction(AUTOSAVE_STORE_NAME, "readonly");
      const request = tx.objectStore(AUTOSAVE_STORE_NAME).get(id);
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  } finally {
    db.close();
  }
}

async function putAutosavedBuffer(buffer) {
  const db = await openAutosaveDb();
  try {
    await new Promise((resolve, reject) => {
      const tx = db.transaction(AUTOSAVE_STORE_NAME, "readwrite");
      tx.objectStore(AUTOSAVE_STORE_NAME).put(buffer);
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
      tx.onabort = () => reject(tx.error);
    });
  } finally {
    db.close();
  }
}

function reportAutosaveError(error) {
  console.warn("ze2 web autosave failed", error);
}

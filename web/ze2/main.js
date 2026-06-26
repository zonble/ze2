const term = new Terminal({
  convertEol: true,
  cursorBlink: true,
  fontFamily: 'Consolas, "Cascadia Mono", "SFMono-Regular", monospace',
  fontSize: 14,
});
const fitAddon = new FitAddon.FitAddon();
term.loadAddon(fitAddon);
term.open(document.getElementById("terminal"));

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

let escFlushTimer = 0;

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
}

function resize() {
  fitAddon.fit();
  api.ze2_web_resize(term.cols, term.rows);
  term.clear();
  flush();
}

fitAddon.fit();
if (!api.ze2_web_init(term.cols, term.rows)) {
  term.writeln("failed to initialize ze2_web.wasm");
} else {
  flush();
}

term.onData((data) => {
  sendInput(data);
});

window.addEventListener("resize", resize);

document.getElementById("open").addEventListener("click", async () => {
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
});

document.getElementById("save").addEventListener("click", () => {
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
});

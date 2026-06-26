import { createReadStream, existsSync } from "node:fs";
import { extname, join, normalize } from "node:path";
import { createServer } from "node:http";

const root = process.cwd();
const port = Number(process.env.PORT || 8080);

const mime = new Map([
  [".html", "text/html; charset=utf-8"],
  [".js", "text/javascript; charset=utf-8"],
  [".css", "text/css; charset=utf-8"],
  [".wasm", "application/wasm"],
]);

createServer((req, res) => {
  const url = new URL(req.url || "/", `http://${req.headers.host || "localhost"}`);
  const requested = url.pathname === "/" ? "/index.html" : decodeURIComponent(url.pathname);
  const path = normalize(join(root, requested));

  if (!path.startsWith(root) || !existsSync(path)) {
    res.writeHead(404);
    res.end("not found");
    return;
  }

  res.writeHead(200, {
    "Content-Type": mime.get(extname(path)) || "application/octet-stream",
  });
  createReadStream(path).pipe(res);
}).listen(port, "127.0.0.1", () => {
  console.log(`ze2 web POC: http://127.0.0.1:${port}/`);
});

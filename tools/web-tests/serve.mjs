import path from "node:path";
import httpServer from "http-server";

const [directory, portText, ...extra] = process.argv.slice(2);
const port = Number(portText);
if (
  !directory ||
  !portText ||
  extra.length !== 0 ||
  !Number.isInteger(port) ||
  port < 1 ||
  port > 65535
) {
  console.error("Usage: node tools/web-tests/serve.mjs DIRECTORY PORT");
  process.exitCode = 1;
} else {
  // The pinned tool's CLI unnecessarily enumerates network interfaces. Bind
  // directly to loopback so serving also works in restricted environments.
  const server = httpServer.createServer({
    root: path.resolve(directory),
    cache: -1,
    cors: false,
  });
  server.server.on("error", (error) => {
    console.error(`Static server failed: ${error.message}`);
    process.exitCode = 1;
  });
  server.listen(port, "127.0.0.1", () => {
    console.log(
      `Serving ${path.resolve(directory)} at http://127.0.0.1:${port}/`,
    );
  });
}

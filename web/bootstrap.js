// The Rust app owns every screen. This host only handles startup failures.
const status = document.querySelector("#startup-status");
const canvas = document.querySelector("#game-canvas");
let ready = false;

// Preserve browser shortcuts and pinch zoom while retaining canvas navigation.
canvas.addEventListener("keydown", (event) => {
  if (
    !event.ctrlKey &&
    !event.metaKey &&
    ["Tab", "Enter", " ", "Escape"].includes(event.key)
  ) {
    event.preventDefault();
  }
});
canvas.addEventListener("pointerdown", () => {
  canvas.focus({ preventScroll: true });
});

function showFailure(error) {
  if (ready) return;
  clearTimeout(startupTimeout);
  const message = error instanceof Error ? error.message : String(error);
  status.replaceChildren();
  const summary = document.createElement("p");
  summary.textContent =
    "The application could not start. Use a browser with WebGL2 enabled, serve this folder over HTTP, and reload.";
  const detail = document.createElement("p");
  detail.textContent = message;
  status.append(summary, detail);
  status.hidden = false;
  status.setAttribute("role", "alert");
  console.error("Low Kingdoms startup failed:", error);
}

window.addEventListener(
  "low-kingdoms-ready",
  () => {
    ready = true;
    clearTimeout(startupTimeout);
    status.hidden = true;
    canvas.focus({ preventScroll: true });
  },
  { once: true },
);

const startupTimeout = setTimeout(() => {
  showFailure(new Error("No application readiness signal within 60 seconds."));
}, 60_000);

try {
  const { default: initialize } = await import("./pkg/low_kingdoms.js");
  await initialize();
} catch (error) {
  showFailure(error);
}

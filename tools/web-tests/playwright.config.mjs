import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "@playwright/test";

const repositoryRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../..",
);
const evidence = path.resolve(
  process.env.LOW_KINGDOMS_EVIDENCE_DIR ??
    path.join(repositoryRoot, "evidence/web"),
);

if (!process.env.LOW_KINGDOMS_BASE_URL) {
  throw new Error("Set LOW_KINGDOMS_BASE_URL or run cargo xtask smoke-web.");
}
if (!process.env.LOW_KINGDOMS_SUBPATH_URL) {
  throw new Error(
    "Set LOW_KINGDOMS_SUBPATH_URL to the real static subpath copy.",
  );
}
if (!process.env.LOW_KINGDOMS_PRODUCTION_URL) {
  throw new Error(
    "Set LOW_KINGDOMS_PRODUCTION_URL to a build without test hooks.",
  );
}

export default defineConfig({
  testDir: "./tests",
  outputDir: path.join(evidence, "results"),
  fullyParallel: false,
  workers: 1,
  retries: 0,
  timeout: 120_000,
  expect: { timeout: 60_000 },
  reporter: [
    ["list"],
    ["json", { outputFile: path.join(evidence, "report.json") }],
    ["html", { outputFolder: path.join(evidence, "report"), open: "never" }],
  ],
  use: {
    browserName: "chromium",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    launchOptions: {
      args: ["--use-gl=angle", "--use-angle=swiftshader", "--enable-webgl"],
    },
  },
  projects: [
    { name: "desktop-720", use: { viewport: { width: 1280, height: 720 } } },
    { name: "desktop-800", use: { viewport: { width: 1280, height: 800 } } },
    {
      name: "phone-portrait",
      use: {
        viewport: { width: 390, height: 844 },
        hasTouch: true,
        isMobile: true,
      },
    },
    {
      name: "phone-landscape-dpr2",
      use: {
        viewport: { width: 844, height: 390 },
        hasTouch: true,
        isMobile: true,
        deviceScaleFactor: 2,
      },
    },
  ],
});

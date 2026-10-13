import { readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";

const projects = [
  "desktop-720",
  "desktop-800",
  "phone-portrait",
  "phone-minimum",
  "phone-landscape-dpr2",
];

function cases(report) {
  const found = [];
  function visit(suites) {
    for (const suite of suites ?? []) {
      for (const spec of suite.specs ?? []) {
        for (const test of spec.tests ?? []) {
          found.push({ id: spec.id, title: spec.title, test });
        }
      }
      visit(suite.suites);
    }
  }
  visit(report.suites);
  return found;
}

function identity(entry) {
  if (!entry.id || !entry.test.projectName) {
    throw new Error("Browser report contains an unidentified case.");
  }
  return JSON.stringify([entry.test.projectName, entry.id]);
}

async function diagnostics(directory) {
  const found = new Map();
  async function visit(current) {
    for (const entry of await readdir(current, { withFileTypes: true })) {
      const file = path.join(current, entry.name);
      if (entry.isSymbolicLink()) {
        throw new Error("Browser evidence must not contain symlinks.");
      }
      if (entry.isDirectory()) {
        await visit(file);
      } else if (
        entry.isFile() &&
        /^(battle-browser-errors|browser-errors|storage-diagnostics)-.*\.json$/.test(
          entry.name,
        )
      ) {
        if (found.has(entry.name)) {
          throw new Error("Browser evidence contains duplicate diagnostics.");
        }
        const value = JSON.parse(await readFile(file, "utf8"));
        const errors = Array.isArray(value) ? value : value.errors;
        if (!Array.isArray(errors) || errors.length !== 0) {
          throw new Error(
            "Browser evidence contains application/browser errors.",
          );
        }
        found.set(entry.name, file);
      }
    }
  }
  await visit(directory);
  return found;
}

// Validate actual results against the inventory discovered from the same source commit.
// This gate never replaces or weakens the maintained real-input test assertions.
export async function verifyReports(inventory, directory) {
  const expected = cases(inventory);
  if (
    expected.length !== 55 ||
    projects.some(
      (project) =>
        expected.filter((entry) => entry.test.projectName === project)
          .length !== 11,
    ) ||
    expected.some((entry) => entry.test.expectedStatus !== "passed")
  ) {
    throw new Error(
      "Expected exactly 55 enabled cases across all five viewports.",
    );
  }
  const pending = new Set(expected.map(identity));
  if (pending.size !== 55) {
    throw new Error("Browser inventory contains duplicate case identities.");
  }
  const summary = { passed: 0, attempts: 0, errorCollections: 0, projects: [] };
  for (const project of projects) {
    const root = path.join(directory, "browser-evidence-" + project);
    const report = JSON.parse(
      await readFile(path.join(root, "report.json"), "utf8"),
    );
    const actual = cases(report);
    if (
      actual.length !== 11 ||
      report.stats?.expected !== 11 ||
      ["unexpected", "flaky", "skipped"].some(
        (field) => report.stats?.[field] !== 0,
      ) ||
      (report.errors ?? []).length !== 0
    ) {
      throw new Error("Incomplete or unsuccessful browser report: " + project);
    }
    const errors = await diagnostics(path.join(root, "results"));
    if (errors.size !== 11) {
      throw new Error("Missing browser diagnostics: " + project);
    }
    for (const entry of actual) {
      const test = entry.test;
      const result = test.results?.[0];
      if (
        test.projectName !== project ||
        test.expectedStatus !== "passed" ||
        test.status !== "expected" ||
        test.results?.length !== 1 ||
        result?.status !== "passed" ||
        result.retry !== 0 ||
        !Array.isArray(result.errors) ||
        result.errors.length !== 0 ||
        !pending.delete(identity(entry))
      ) {
        throw new Error(
          "Failed, repeated, or unexpected browser case: " + project,
        );
      }
      const attached = (result.attachments ?? []).filter((attachment) =>
        errors.has(path.basename(attachment.path ?? "")),
      );
      if (attached.length !== 1) {
        throw new Error("Case has no unique browser diagnostics: " + project);
      }
      errors.delete(path.basename(attached[0].path));
      summary.passed += 1;
      summary.attempts += 1;
      summary.errorCollections += 1;
    }
    summary.projects.push({
      name: project,
      passed: actual.length,
      durationMs: report.stats.duration,
    });
  }
  if (pending.size !== 0) {
    throw new Error("Browser matrix omitted required cases.");
  }
  return summary;
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  try {
    const [, , inventoryFile, directory, output] = process.argv;
    if (!inventoryFile || !directory || !output || process.argv.length !== 5) {
      throw new Error(
        "Usage: verify-reports.mjs INVENTORY REPORT_DIRECTORY OUTPUT",
      );
    }
    const inventory = JSON.parse(await readFile(inventoryFile, "utf8"));
    const summary = await verifyReports(inventory, directory);
    await writeFile(output, JSON.stringify(summary, null, 2) + "\n");
    console.log(
      "Verified all 55 browser cases and 55 empty error collections.",
    );
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}

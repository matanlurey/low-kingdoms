import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

import { verifyReports } from "./verify-reports.mjs";

const projects = [
  "desktop-720",
  "desktop-800",
  "phone-portrait",
  "phone-minimum",
  "phone-landscape-dpr2",
];

async function fixture(t) {
  const directory = await mkdtemp(path.join(tmpdir(), "low-kingdoms-reports-"));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const inventory = { suites: [{ specs: [] }] };
  const reports = [];
  for (const project of projects) {
    const root = path.join(directory, "browser-evidence-" + project);
    const resultDirectory = path.join(root, "results");
    await mkdir(resultDirectory, { recursive: true });
    const report = {
      errors: [],
      stats: {
        expected: 11,
        unexpected: 0,
        flaky: 0,
        skipped: 0,
        duration: 100,
      },
      suites: [{ specs: [] }],
    };
    for (let index = 0; index < 11; index += 1) {
      const id = project + "-" + index;
      const prefix =
        index < 4
          ? "battle-browser-errors"
          : index < 7
            ? "browser-errors"
            : "storage-diagnostics";
      const diagnostic = path.join(
        resultDirectory,
        prefix + "-" + id + ".json",
      );
      await writeFile(
        diagnostic,
        JSON.stringify(index < 7 ? [] : { errors: [] }),
      );
      const listed = {
        id,
        title: "case " + index,
        tests: [
          { projectName: project, expectedStatus: "passed", results: [] },
        ],
      };
      inventory.suites[0].specs.push(listed);
      report.suites[0].specs.push({
        id,
        title: listed.title,
        tests: [
          {
            projectName: project,
            expectedStatus: "passed",
            status: "expected",
            results: [
              {
                status: "passed",
                retry: 0,
                errors: [],
                attachments: [{ path: diagnostic }],
              },
            ],
          },
        ],
      });
    }
    const file = path.join(root, "report.json");
    await writeFile(file, JSON.stringify(report));
    reports.push({ file, report });
  }
  return { directory, inventory, reports };
}

test("requires the complete five-viewport matrix with one attempt per case", async (t) => {
  const data = await fixture(t);
  const summary = await verifyReports(data.inventory, data.directory);
  assert.equal(summary.passed, 55);
  assert.equal(summary.attempts, 55);
  assert.equal(summary.errorCollections, 55);
  assert.deepEqual(
    summary.projects.map((p) => p.name),
    projects,
  );
});

test("rejects a missing viewport report", async (t) => {
  const data = await fixture(t);
  await rm(data.reports[4].file);
  await assert.rejects(verifyReports(data.inventory, data.directory), /ENOENT/);
});

test("rejects missing, duplicate, and unknown cases despite successful totals", async (t) => {
  for (const kind of ["missing", "duplicate", "unknown"]) {
    const data = await fixture(t);
    const first = data.reports[0];
    const specs = first.report.suites[0].specs;
    if (kind === "missing") specs.pop();
    if (kind === "duplicate") specs[1].id = specs[0].id;
    if (kind === "unknown") specs[0].id = "unlisted-case";
    await writeFile(first.file, JSON.stringify(first.report));
    await assert.rejects(verifyReports(data.inventory, data.directory));
  }
});

test("rejects skipped, flaky, failed, and retried outcomes", async (t) => {
  for (const kind of [
    "skipped",
    "flaky",
    "failed",
    "retried",
    "retry-number",
  ]) {
    const data = await fixture(t);
    const first = data.reports[0];
    const actual = first.report.suites[0].specs[0].tests[0];
    if (kind === "skipped") first.report.stats.skipped = 1;
    if (kind === "flaky") first.report.stats.flaky = 1;
    if (kind === "failed") actual.results[0].status = "failed";
    if (kind === "retried")
      actual.results.push(structuredClone(actual.results[0]));
    if (kind === "retry-number") actual.results[0].retry = 1;
    await writeFile(first.file, JSON.stringify(first.report));
    await assert.rejects(verifyReports(data.inventory, data.directory));
  }
});

test("rejects absent or nonempty diagnostic attachments", async (t) => {
  for (const kind of ["missing", "unattached", "nonempty"]) {
    const data = await fixture(t);
    const first = data.reports[0];
    const result = first.report.suites[0].specs[0].tests[0].results[0];
    const diagnostic = result.attachments[0].path;
    if (kind === "missing") await rm(diagnostic);
    if (kind === "unattached") {
      result.attachments = [];
      await writeFile(first.file, JSON.stringify(first.report));
    }
    if (kind === "nonempty")
      await writeFile(diagnostic, '["application error"]');
    await assert.rejects(verifyReports(data.inventory, data.directory));
  }
});

test("rejects an incomplete or disabled inventory", async (t) => {
  const data = await fixture(t);
  const original = structuredClone(data.inventory);
  data.inventory.suites[0].specs.pop();
  await assert.rejects(
    verifyReports(data.inventory, data.directory),
    /55 enabled/,
  );
  original.suites[0].specs[0].tests[0].expectedStatus = "skipped";
  await assert.rejects(verifyReports(original, data.directory), /55 enabled/);
});

test("rejects mismatched report statistics and infrastructure errors", async (t) => {
  const data = await fixture(t);
  const first = data.reports[0];
  first.report.stats.expected = 10;
  await writeFile(first.file, JSON.stringify(first.report));
  await assert.rejects(verifyReports(data.inventory, data.directory));
  first.report.stats.expected = 11;
  first.report.errors.push({ message: "worker failed" });
  await writeFile(first.file, JSON.stringify(first.report));
  await assert.rejects(verifyReports(data.inventory, data.directory));
});

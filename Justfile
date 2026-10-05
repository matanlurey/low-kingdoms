# Node executes argument arrays directly; no platform shell interpolation.

set shell := ["node", "-e"]
set positional-arguments := true

runner := "function run(command) { const result = require('node:child_process').spawnSync('cargo', ['xtask', command, ...process.argv.slice(2)], { stdio: 'inherit' }); if (result.error) console.error(result.error.message); process.exit(result.status ?? 1); }"

default:
    console.log("Run just --list to show available commands; cargo xtask also works directly.")

doctor *args:
    {{ runner }} run("doctor");

setup *args:
    {{ runner }} run("setup");

dev *args:
    {{ runner }} run("dev");

dev-web *args:
    {{ runner }} run("dev-web");

build *args:
    {{ runner }} run("build");

build-web *args:
    {{ runner }} run("build-web");

fmt *args:
    {{ runner }} run("fmt");

fmt-check *args:
    {{ runner }} run("fmt-check");

clippy *args:
    {{ runner }} run("clippy");

test *args:
    {{ runner }} run("test");

doc *args:
    {{ runner }} run("doc");

check *args:
    {{ runner }} run("check");

check-web *args:
    {{ runner }} run("check-web");

smoke-native *args:
    {{ runner }} run("smoke-native");

smoke-web *args:
    {{ runner }} run("smoke-web");

check-all *args:
    {{ runner }} run("check-all");

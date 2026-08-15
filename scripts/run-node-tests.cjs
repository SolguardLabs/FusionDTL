"use strict";

const { spawnSync } = require("node:child_process");
const { readdirSync } = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const directory = path.join(root, "tests", "node");
const files = readdirSync(directory)
    .filter((name) => name.endsWith(".test.js"))
    .sort()
    .map((name) => path.join(directory, name));

if (files.length === 0) throw new Error("No Node test files were found");

const result = spawnSync(process.execPath, ["--test", ...files], {
    cwd: root,
    encoding: "utf8",
    stdio: "inherit",
    timeout: 180_000,
});
if (result.error) throw result.error;
process.exit(result.status ?? 1);

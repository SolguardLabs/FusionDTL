"use strict";

const { spawnSync } = require("node:child_process");
const { readdirSync, statSync } = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const directories = ["scripts", "sdk", "tests/helpers", "tests/node"];

function collect(directory) {
    const files = [];
    for (const name of readdirSync(path.join(root, directory))) {
        const relative = path.join(directory, name);
        const details = statSync(path.join(root, relative));
        if (details.isDirectory()) files.push(...collect(relative));
        else if (details.isFile() && /\.(?:c?js)$/.test(name)) files.push(relative);
    }
    return files;
}

const files = directories.flatMap(collect).sort();
for (const file of files) {
    const result = spawnSync(process.execPath, ["--check", file], {
        cwd: root,
        encoding: "utf8",
        stdio: "pipe",
    });
    if (result.status !== 0) {
        process.stderr.write(result.stderr);
        process.stdout.write(result.stdout);
        process.exit(result.status ?? 1);
    }
}
console.log(`Sintaxis JavaScript verificada en ${files.length} archivos.`);

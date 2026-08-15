"use strict";

const assert = require("node:assert/strict");
const { createHash } = require("node:crypto");
const { readFileSync, readdirSync, statSync } = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const protectedFiles = new Map([
    ["src/ledger/state.rs", "3A8E8C81210C3A8ABAE22BC0FFA79899DF48E56529F3B504B3EB95D92C68DB81"],
    ["src/delivery/packet.rs", "EFC67F7FE45484FE9C8EA0EA705D3C31131BF2EC77A2FF236A00A9DEA4960F6E"],
    ["src/routing/lane.rs", "3ADE200A4598ED002347BB6C1DE2BC90ACBF655A4614827EA8FA08FBB955DDCF"],
    ["src/fusion/cell.rs", "D1CC09D2C5887F56FD83ED9A25E97E821673434C7BA0C1B70B92F931A20F7E82"],
]);
const bannerHash = "6F63A6D11DFE406733DFA7393DC98F61BB2B19B68D72D6B647028B24EE0D6D03";

function sha256(file) {
    return createHash("sha256")
        .update(readFileSync(path.join(root, file)))
        .digest("hex")
        .toUpperCase();
}

function sha256Text(file) {
    const content = readFileSync(path.join(root, file), "utf8").replace(/\r\n/g, "\n");
    return createHash("sha256").update(content).digest("hex").toUpperCase();
}

function collect(directory) {
    const files = [];
    for (const name of readdirSync(path.join(root, directory))) {
        if ([".git", "node_modules", "target", "private"].includes(name)) continue;
        const relative = path.join(directory, name);
        const details = statSync(path.join(root, relative));
        files.push(...(details.isDirectory() ? collect(relative) : [relative]));
    }
    return files;
}

for (const [file, expected] of protectedFiles) {
    assert.equal(sha256Text(file), expected, `${file} changed from the reviewed economic baseline`);
}
assert.equal(sha256("assets/banner.png"), bannerHash, "release banner changed unexpectedly");

const cargo = readFileSync(path.join(root, "Cargo.toml"), "utf8");
const pkg = JSON.parse(readFileSync(path.join(root, "package.json"), "utf8"));
assert.match(cargo, /^version = "1\.0\.0"$/m);
assert.equal(pkg.version, "1.0.0");

const docs = readdirSync(path.join(root, "docs")).filter((name) => name.endsWith(".md"));
assert.equal(docs.length, 7, "docs/ must contain seven operational guides");
const markdown = ["README.md", "SECURITY.md", ...docs.map((name) => path.join("docs", name))];
const mermaidCount = markdown.reduce(
    (total, file) =>
        total + (readFileSync(path.join(root, file), "utf8").match(/```mermaid/g) ?? []).length,
    0,
);
assert.equal(mermaidCount, 27, "the documentation set must contain 27 Mermaid diagrams");

const restrictedWords = [
    "c" + "tf",
    "la" + "b",
    "la" + "bs",
    "labor" + "atorio",
    "labor" + "atorios",
    "vulnera" + "bilidad",
    "vulnera" + "bilidades",
    "vulnera" + "ble",
    "bu" + "g",
    "bu" + "gs",
    "explo" + "it",
    "explo" + "itar",
    "by" + "pass",
    "att" + "acker",
    "atac" + "ante",
];
const restricted = new RegExp(`\\b(?:${restrictedWords.join("|")})\\b`, "iu");
const publicExtensions = new Set([
    ".cjs",
    ".js",
    ".json",
    ".md",
    ".rs",
    ".sh",
    ".toml",
    ".yaml",
    ".yml",
]);
for (const file of collect(".")) {
    if (!publicExtensions.has(path.extname(file))) continue;
    assert.equal(
        restricted.test(readFileSync(path.join(root, file), "utf8")),
        false,
        `restricted public wording found in ${file}`,
    );
}

const rustFiles = collect("src").filter((file) => file.endsWith(".rs"));
const rustLoc = rustFiles.reduce(
    (total, file) => total + readFileSync(path.join(root, file), "utf8").split(/\r?\n/).length,
    0,
);
const rustTests = collect("src")
    .concat(collect("tests"))
    .filter((file) => file.endsWith(".rs"))
    .reduce(
        (total, file) =>
            total + (readFileSync(path.join(root, file), "utf8").match(/#\[test\]/g) ?? []).length,
        0,
    );
const nodeTests = collect("tests/node")
    .filter((file) => file.endsWith(".test.js"))
    .reduce(
        (total, file) =>
            total + (readFileSync(path.join(root, file), "utf8").match(/^test\(/gm) ?? []).length,
        0,
    );
assert.ok(rustLoc >= 3_500, `expected at least 3500 Rust lines, received ${rustLoc}`);
assert.ok(rustTests >= 17, `expected at least 17 Rust tests, received ${rustTests}`);
assert.ok(nodeTests >= 33, `expected at least 33 Node tests, received ${nodeTests}`);

console.log(
    `Release verificada: ${rustLoc} lineas Rust, ${rustTests} tests Rust, ${nodeTests} tests Node, ${mermaidCount} diagramas.`,
);

const { spawnSync } = require("node:child_process");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..", "..");
const expectedSupply = 140_000_000_000;
const scenarioNames = ["snapshot", "issue", "settle", "rebalance"];

function runScenario(name) {
    const args = ["run", "--quiet"];
    if (name) {
        args.push("--", name);
    }

    const result = spawnSync("cargo", args, {
        cwd: repoRoot,
        encoding: "utf8",
    });

    if (result.status !== 0) {
        throw new Error(
            [
                `cargo ${args.join(" ")} failed with status ${result.status}`,
                `stdout:\n${result.stdout}`,
                `stderr:\n${result.stderr}`,
            ].join("\n"),
        );
    }

    return JSON.parse(result.stdout);
}

function runScenarios(names = scenarioNames) {
    return Object.fromEntries(names.map((name) => [name, runScenario(name)]));
}

function observedSupply(report) {
    const balances = Object.values(report.balances).reduce((total, value) => total + value, 0);
    return balances + report.cells.core_reserve + report.cells.edge_reserve;
}

function isDigest(value) {
    return typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
}

function assertDigest(assert, value, label) {
    assert.equal(isDigest(value), true, `${label} debe ser un digest hexadecimal de 32 bytes`);
}

function assertNonNegativeInteger(assert, value, label) {
    assert.equal(Number.isInteger(value), true, `${label} debe ser entero`);
    assert.equal(value >= 0, true, `${label} no puede ser negativo`);
}

function assertBaseSurface(assert, report) {
    assert.equal(report.network_id, 61_804);
    assert.equal(report.asset.symbol, "FUSD");
    assert.equal(report.asset.decimals, 6);
    assert.equal(report.surface.oracle_markets, 1);
    assert.equal(report.surface.participant_profiles, 6);
    assert.equal(report.surface.active_profiles, 6);
    assert.equal(report.surface.settlement_windows, 1);
    assert.equal(report.surface.operators, 5);
    assert.equal(report.surface.role_assignments, 7);
    assert.equal(report.surface.delivery_lanes, 2);
    assert.equal(report.surface.relayer_quotes, 2);
    assert.equal(report.surface.capacity_policies, 2);
    assert.equal(report.conservation_ok, true);
}

module.exports = {
    assertDigest,
    assertBaseSurface,
    assertNonNegativeInteger,
    expectedSupply,
    isDigest,
    observedSupply,
    runScenarios,
    runScenario,
    scenarioNames,
};

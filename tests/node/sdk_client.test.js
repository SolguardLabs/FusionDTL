const test = require("node:test");
const assert = require("node:assert/strict");

const { FusionClient } = require("../../sdk/client");
const { normalizeReport, operationalSnapshot } = require("../../sdk/report");

test("client returns a validated settlement report", () => {
    const report = new FusionClient().run("settle");
    assert.equal(report.scenario, "settle");
    assert.equal(report.conservation_ok, true);
    assert.equal(report.transactions.length, 2);
});

test("client exposes a deterministic snapshot", () => {
    const snapshot = new FusionClient().snapshot("snapshot");
    assert.equal(snapshot.status, "reconciled");
    assert.equal(snapshot.observedSupply, 140_000_000_000);
    assert.equal(snapshot.activeParticipantRatioBps, 10_000);
});

test("client can execute the routed flow", () => {
    const report = new FusionClient().run("rebalance");
    assert.equal(report.cells.core_reserve, 87_500_000_000);
    assert.equal(report.surface.processed_packets, 1);
    assert.equal(report.transactions.length, 3);
});

test("client validates timeout configuration", () => {
    assert.throws(() => new FusionClient({ timeoutMs: 0 }), /positive integer/);
});

test("report normalization rejects missing state", () => {
    assert.throws(() => normalizeReport({ scenario: "snapshot" }), /asset is required/);
});

test("operational snapshot reports pending liabilities", () => {
    const report = new FusionClient().run("issue");
    const snapshot = operationalSnapshot(report);
    assert.equal(snapshot.pendingLiabilities, 2_500_000_000);
    assert.equal(snapshot.cellReserves, 92_500_000_000);
});

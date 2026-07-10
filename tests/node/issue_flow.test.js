const test = require("node:test");
const assert = require("node:assert/strict");

const {
    assertBaseSurface,
    expectedSupply,
    observedSupply,
    runScenario,
} = require("../helpers/fusionCli");

test("issue bloquea importe y registra exposición de celda", () => {
    const report = runScenario("issue");

    assert.equal(report.scenario, "issue");
    assert.equal(report.receipt.amount, 2_500_000_000);
    assert.equal(report.balances.issuer, 17_500_000_000);
    assert.equal(report.cells.edge_reserve, 2_500_000_000);
    assert.equal(report.cells.edge_pending, 2_500_000_000);
    assert.equal(report.surface.receipts, 1);
    assert.equal(report.surface.processed_packets, 0);
    assert.equal(report.surface.exposure_cells, 1);
    assert.equal(report.transactions.length, 1);
    assert.equal(observedSupply(report), expectedSupply);
    assertBaseSurface(assert, report);
});

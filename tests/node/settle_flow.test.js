const test = require("node:test");
const assert = require("node:assert/strict");

const {
    assertBaseSurface,
    expectedSupply,
    observedSupply,
    runScenario,
} = require("../helpers/fusionCli");

test("settle liquida beneficiario, relayer y reservas", () => {
    const report = runScenario("settle");

    assert.equal(report.scenario, "settle");
    assert.equal(report.balances.issuer, 17_500_000_000);
    assert.equal(report.balances.beneficiary, 2_495_000_000);
    assert.equal(report.balances.relayer, 5_000_000);
    assert.equal(report.cells.edge_reserve, 0);
    assert.equal(report.cells.edge_pending, 0);
    assert.equal(report.surface.receipts, 1);
    assert.equal(report.surface.processed_packets, 1);
    assert.equal(report.surface.treasury_assets, 1);
    assert.equal(report.transactions.length, 2);
    assert.equal(observedSupply(report), expectedSupply);
    assertBaseSurface(assert, report);
});

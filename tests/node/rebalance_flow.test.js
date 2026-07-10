const test = require("node:test");
const assert = require("node:assert/strict");

const {
    assertBaseSurface,
    expectedSupply,
    observedSupply,
    runScenario,
} = require("../helpers/fusionCli");

test("rebalance liquida por celda core y conserva los saldos", () => {
    const report = runScenario("rebalance");

    assert.equal(report.scenario, "rebalance");
    assert.equal(report.balances.beneficiary, 2_495_000_000);
    assert.equal(report.balances.relayer, 5_000_000);
    assert.equal(report.balances.edge_controller, 2_500_000_000);
    assert.equal(report.cells.core_reserve, 87_500_000_000);
    assert.equal(report.cells.edge_reserve, 0);
    assert.equal(report.cells.edge_pending, 0);
    assert.equal(report.surface.exposure_cells, 2);
    assert.equal(report.transactions.length, 3);
    assert.equal(observedSupply(report), expectedSupply);
    assertBaseSurface(assert, report);
});

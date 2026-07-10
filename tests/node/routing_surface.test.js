const test = require("node:test");
const assert = require("node:assert/strict");

const { runScenarios } = require("../helpers/fusionCli");

test("settle usa la liquidez edge y rebalance usa la liquidez core", () => {
    const { settle, rebalance } = runScenarios(["settle", "rebalance"]);

    assert.equal(settle.cells.edge_reserve, 0);
    assert.equal(settle.cells.core_reserve, 90_000_000_000);
    assert.equal(rebalance.cells.edge_reserve, 0);
    assert.equal(rebalance.cells.core_reserve, 87_500_000_000);
});

test("la exposición distingue liquidación local y liquidación enrutada", () => {
    const { settle, rebalance } = runScenarios(["settle", "rebalance"]);

    assert.equal(settle.surface.exposure_cells, 1);
    assert.equal(rebalance.surface.exposure_cells, 2);
});

test("la comisión de relayer es constante en las rutas operativas", () => {
    const { settle, rebalance } = runScenarios(["settle", "rebalance"]);

    assert.equal(settle.balances.relayer, 5_000_000);
    assert.equal(rebalance.balances.relayer, 5_000_000);
    assert.equal(settle.balances.beneficiary, rebalance.balances.beneficiary);
});

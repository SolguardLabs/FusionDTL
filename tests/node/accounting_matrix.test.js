const test = require("node:test");
const assert = require("node:assert/strict");

const {
    assertBaseSurface,
    expectedSupply,
    observedSupply,
    runScenarios,
} = require("../helpers/fusionCli");

test("todos los escenarios conservan el suministro observado", () => {
    const reports = runScenarios();

    for (const [name, report] of Object.entries(reports)) {
        assert.equal(observedSupply(report), expectedSupply, `${name} conserva el suministro`);
        assertBaseSurface(assert, report);
    }
});

test("issue mueve exactamente el importe emitido desde issuer a la celda edge", () => {
    const { snapshot, issue } = runScenarios(["snapshot", "issue"]);
    const issuedAmount = issue.receipt.amount;

    assert.equal(snapshot.balances.issuer - issue.balances.issuer, issuedAmount);
    assert.equal(issue.cells.edge_reserve - snapshot.cells.edge_reserve, issuedAmount);
    assert.equal(issue.cells.edge_pending - snapshot.cells.edge_pending, issuedAmount);
});

test("settle distribuye el importe entre beneficiario y relayer", () => {
    const { settle } = runScenarios(["settle"]);
    const grossAmount = settle.receipt.amount;
    const beneficiaryAmount = settle.balances.beneficiary;
    const relayerFee = settle.balances.relayer;

    assert.equal(beneficiaryAmount + relayerFee, grossAmount);
    assert.equal(relayerFee, 5_000_000);
});

test("rebalance mantiene cerrada la obligación de edge tras la liquidación", () => {
    const { rebalance } = runScenarios(["rebalance"]);

    assert.equal(rebalance.cells.edge_pending, 0);
    assert.equal(rebalance.cells.edge_reserve, 0);
    assert.equal(rebalance.surface.processed_packets, 1);
});

const test = require("node:test");
const assert = require("node:assert/strict");

const { runScenarios } = require("../helpers/fusionCli");

test("el journal crece por transición de negocio ejecutada", () => {
    const reports = runScenarios();

    assert.equal(reports.snapshot.journal_entries, 28);
    assert.equal(reports.issue.journal_entries, reports.snapshot.journal_entries + 1);
    assert.equal(reports.settle.journal_entries, reports.issue.journal_entries + 1);
    assert.equal(reports.rebalance.journal_entries, reports.settle.journal_entries + 1);
});

test("los contadores de superficie avanzan con cada flujo", () => {
    const reports = runScenarios();

    assert.equal(reports.snapshot.surface.receipts, 0);
    assert.equal(reports.issue.surface.receipts, 1);
    assert.equal(reports.settle.surface.processed_packets, 1);
    assert.equal(reports.rebalance.surface.processed_packets, 1);
    assert.equal(reports.snapshot.surface.treasury_assets, 0);
    assert.equal(reports.settle.surface.treasury_assets, 1);
    assert.equal(reports.rebalance.surface.exposure_cells, 2);
});

test("las rutas de liquidación generan distinto número de transacciones externas", () => {
    const reports = runScenarios(["issue", "settle", "rebalance"]);

    assert.equal(reports.issue.transactions.length, 1);
    assert.equal(reports.settle.transactions.length, 2);
    assert.equal(reports.rebalance.transactions.length, 3);
});

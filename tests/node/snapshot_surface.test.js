const test = require("node:test");
const assert = require("node:assert/strict");

const {
    assertBaseSurface,
    expectedSupply,
    observedSupply,
    runScenario,
} = require("../helpers/fusionCli");

test("snapshot expone la superficie operativa inicial", () => {
    const report = runScenario("snapshot");

    assert.equal(report.scenario, "snapshot");
    assert.equal(report.receipt, null);
    assert.equal(report.surface.receipts, 0);
    assert.equal(report.surface.processed_packets, 0);
    assert.equal(report.surface.exposure_cells, 0);
    assert.equal(report.journal_entries, 28);
    assert.equal(observedSupply(report), expectedSupply);
    assertBaseSurface(assert, report);
});

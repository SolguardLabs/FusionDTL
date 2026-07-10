const test = require("node:test");
const assert = require("node:assert/strict");

const { assertBaseSurface, runScenario } = require("../helpers/fusionCli");

test("el escenario por defecto ejecuta settle", () => {
    const report = runScenario();

    assert.equal(report.scenario, "settle");
    assert.equal(report.surface.processed_packets, 1);
    assert.equal(report.transactions.length, 2);
    assertBaseSurface(assert, report);
});

test("un escenario no reconocido usa la ruta settle", () => {
    const report = runScenario("unknown");

    assert.equal(report.scenario, "settle");
    assert.equal(report.surface.processed_packets, 1);
    assert.equal(report.transactions.length, 2);
    assertBaseSurface(assert, report);
});

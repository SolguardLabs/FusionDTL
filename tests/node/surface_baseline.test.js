const test = require("node:test");
const assert = require("node:assert/strict");

const { runScenarios } = require("../helpers/fusionCli");

test("la superficie estática permanece inicializada en todos los escenarios", () => {
    const reports = runScenarios();

    for (const [name, report] of Object.entries(reports)) {
        assert.equal(report.surface.participant_profiles, 6, `${name}.participant_profiles`);
        assert.equal(report.surface.active_profiles, 6, `${name}.active_profiles`);
        assert.equal(report.surface.settlement_windows, 1, `${name}.settlement_windows`);
        assert.equal(report.surface.operators, 5, `${name}.operators`);
        assert.equal(report.surface.role_assignments, 7, `${name}.role_assignments`);
        assert.equal(report.surface.delivery_lanes, 2, `${name}.delivery_lanes`);
        assert.equal(report.surface.relayer_quotes, 2, `${name}.relayer_quotes`);
        assert.equal(report.surface.capacity_policies, 2, `${name}.capacity_policies`);
    }
});

test("los escenarios de ejecución no alteran la metadata del activo", () => {
    const reports = runScenarios();
    const assetIds = Object.values(reports).map((report) => report.asset.id);

    assert.equal(new Set(assetIds).size, 1);
    for (const report of Object.values(reports)) {
        assert.equal(report.asset.symbol, "FUSD");
        assert.equal(report.asset.decimals, 6);
    }
});

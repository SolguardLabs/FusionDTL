const test = require("node:test");
const assert = require("node:assert/strict");

const { assertDigest, runScenarios } = require("../helpers/fusionCli");

test("los recibos emitidos preservan los campos económicos esperados", () => {
    const reports = runScenarios(["issue", "settle", "rebalance"]);

    for (const [name, report] of Object.entries(reports)) {
        assert.equal(report.receipt.network_id, 61_804, `${name}.network_id`);
        assert.equal(report.receipt.amount, 2_500_000_000, `${name}.amount`);
        assert.equal(report.receipt.owner_nonce, 0, `${name}.owner_nonce`);
        assert.equal(report.receipt.maturity_epoch, 900, `${name}.maturity_epoch`);
        assert.equal(report.receipt.asset, report.asset.id, `${name}.asset`);
        assert.equal(report.receipt.receipt_id, report.receipt_id, `${name}.receipt_id`);
    }
});

test("cada recibo conserva un route digest propio del flujo", () => {
    const reports = runScenarios(["issue", "settle", "rebalance"]);
    const routeDigests = Object.values(reports).map((report) => report.receipt.route_digest);

    for (const [index, digest] of routeDigests.entries()) {
        assertDigest(assert, digest, `route_digest.${index}`);
    }
    assert.equal(new Set(routeDigests).size, routeDigests.length);
});

test("snapshot no expone recibo ni receipt_id", () => {
    const { snapshot } = runScenarios(["snapshot"]);

    assert.equal(snapshot.receipt, null);
    assert.equal(snapshot.receipt_id, null);
});

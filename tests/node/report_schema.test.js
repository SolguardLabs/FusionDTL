const test = require("node:test");
const assert = require("node:assert/strict");

const {
    assertDigest,
    assertNonNegativeInteger,
    runScenarios,
    scenarioNames,
} = require("../helpers/fusionCli");

const balanceKeys = [
    "issuer",
    "beneficiary",
    "relayer",
    "core_lp",
    "core_controller",
    "edge_controller",
];

const cellKeys = ["core_reserve", "core_pending", "edge_reserve", "edge_pending"];

test("todos los escenarios mantienen el contrato JSON principal", () => {
    const reports = runScenarios();

    for (const name of scenarioNames) {
        const report = reports[name];

        assert.equal(report.scenario, name);
        assert.equal(typeof report.network_id, "number");
        assert.equal(typeof report.asset, "object");
        assert.equal(typeof report.balances, "object");
        assert.equal(typeof report.cells, "object");
        assert.equal(typeof report.surface, "object");
        assert.equal(Array.isArray(report.transactions), true);
        assertDigest(assert, report.state_digest, `${name}.state_digest`);
    }
});

test("todos los importes expuestos son enteros no negativos", () => {
    const reports = runScenarios();

    for (const [name, report] of Object.entries(reports)) {
        for (const key of balanceKeys) {
            assertNonNegativeInteger(assert, report.balances[key], `${name}.balances.${key}`);
        }
        for (const key of cellKeys) {
            assertNonNegativeInteger(assert, report.cells[key], `${name}.cells.${key}`);
        }
    }
});

test("los identificadores serializados usan formato digest", () => {
    const reports = runScenarios(["issue", "settle", "rebalance"]);

    for (const [name, report] of Object.entries(reports)) {
        assertDigest(assert, report.asset.id, `${name}.asset.id`);
        assertDigest(assert, report.receipt_id, `${name}.receipt_id`);
        assertDigest(assert, report.receipt.receipt_id, `${name}.receipt.receipt_id`);
        assertDigest(assert, report.receipt.beneficiary, `${name}.receipt.beneficiary`);
        assertDigest(assert, report.receipt.asset, `${name}.receipt.asset`);
        assertDigest(assert, report.receipt.route_digest, `${name}.receipt.route_digest`);

        for (const [index, txId] of report.transactions.entries()) {
            assertDigest(assert, txId, `${name}.transactions.${index}`);
        }
    }
});

const test = require("node:test");
const assert = require("node:assert/strict");

const { runScenario, runScenarios, scenarioNames } = require("../helpers/fusionCli");

test("un mismo escenario produce el mismo digest de estado", () => {
    for (const name of scenarioNames) {
        const first = runScenario(name);
        const second = runScenario(name);

        assert.equal(second.state_digest, first.state_digest, `${name} debe ser determinista`);
        assert.deepEqual(second.transactions, first.transactions);
    }
});

test("cada escenario principal termina en un digest distinto", () => {
    const reports = runScenarios();
    const digests = Object.values(reports).map((report) => report.state_digest);
    const uniqueDigests = new Set(digests);

    assert.equal(uniqueDigests.size, digests.length);
});

test("los recibos de flujos distintos no reutilizan identificador", () => {
    const reports = runScenarios(["issue", "settle", "rebalance"]);
    const receiptIds = Object.values(reports).map((report) => report.receipt_id);

    assert.equal(new Set(receiptIds).size, receiptIds.length);
});

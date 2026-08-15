"use strict";

const REQUIRED_SURFACE = [
    "receipts",
    "processed_packets",
    "oracle_markets",
    "participant_profiles",
    "active_profiles",
    "settlement_windows",
    "operators",
    "role_assignments",
    "delivery_lanes",
    "relayer_quotes",
    "treasury_assets",
    "exposure_cells",
    "capacity_policies",
];

function nonNegativeInteger(value, label) {
    if (!Number.isSafeInteger(value) || value < 0) {
        throw new TypeError(`${label} must be a non-negative safe integer`);
    }
    return value;
}

function normalizeReport(value) {
    if (value === null || typeof value !== "object" || Array.isArray(value)) {
        throw new TypeError("FusionDTL report must be an object");
    }
    if (typeof value.scenario !== "string" || value.scenario.length === 0) {
        throw new TypeError("FusionDTL report scenario must be a non-empty string");
    }
    if (value.asset === null || typeof value.asset !== "object") {
        throw new TypeError("FusionDTL report asset is required");
    }
    if (value.balances === null || typeof value.balances !== "object") {
        throw new TypeError("FusionDTL report balances are required");
    }
    if (value.cells === null || typeof value.cells !== "object") {
        throw new TypeError("FusionDTL report cells are required");
    }
    if (value.surface === null || typeof value.surface !== "object") {
        throw new TypeError("FusionDTL report surface is required");
    }
    for (const [name, amount] of Object.entries(value.balances)) {
        nonNegativeInteger(amount, `balances.${name}`);
    }
    for (const [name, amount] of Object.entries(value.cells)) {
        nonNegativeInteger(amount, `cells.${name}`);
    }
    for (const field of REQUIRED_SURFACE) {
        nonNegativeInteger(value.surface[field], `surface.${field}`);
    }
    nonNegativeInteger(value.network_id, "network_id");
    nonNegativeInteger(value.asset.decimals, "asset.decimals");
    nonNegativeInteger(value.journal_entries, "journal_entries");
    if (!/^[0-9a-f]{64}$/.test(value.state_digest)) {
        throw new TypeError("state_digest must be a 32-byte hexadecimal digest");
    }
    if (typeof value.conservation_ok !== "boolean") {
        throw new TypeError("conservation_ok must be boolean");
    }
    return Object.freeze(value);
}

function operationalSnapshot(value) {
    const report = normalizeReport(value);
    const participantBalances = Object.values(report.balances).reduce(
        (total, amount) => total + amount,
        0,
    );
    const cellReserves = report.cells.core_reserve + report.cells.edge_reserve;
    const pendingLiabilities = report.cells.core_pending + report.cells.edge_pending;
    const observedSupply = participantBalances + cellReserves;

    return Object.freeze({
        report,
        observedSupply,
        cellReserves,
        pendingLiabilities,
        activeParticipantRatioBps:
            report.surface.participant_profiles === 0
                ? 0
                : Math.floor(
                      (report.surface.active_profiles * 10_000) /
                          report.surface.participant_profiles,
                  ),
        status: report.conservation_ok ? "reconciled" : "attention",
    });
}

module.exports = { normalizeReport, operationalSnapshot };

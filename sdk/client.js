"use strict";

const { spawnSync } = require("node:child_process");
const path = require("node:path");
const { normalizeReport, operationalSnapshot } = require("./report");

class FusionClient {
    constructor({
        cwd = path.resolve(__dirname, ".."),
        cargo = "cargo",
        timeoutMs = 30_000,
        env = {},
    } = {}) {
        if (!Number.isSafeInteger(timeoutMs) || timeoutMs <= 0) {
            throw new TypeError("timeoutMs must be a positive integer");
        }
        this.cwd = path.resolve(cwd);
        this.cargo = cargo;
        this.timeoutMs = timeoutMs;
        this.env = { ...process.env, CARGO_TERM_COLOR: "never", ...env };
    }

    run(name = "settle") {
        if (typeof name !== "string" || name.length === 0) {
            throw new TypeError("scenario name must be a non-empty string");
        }
        const result = spawnSync(this.cargo, ["run", "--quiet", "--", name], {
            cwd: this.cwd,
            env: this.env,
            encoding: "utf8",
            stdio: ["ignore", "pipe", "pipe"],
            timeout: this.timeoutMs,
            maxBuffer: 4 * 1024 * 1024,
            windowsHide: true,
        });
        if (result.error) throw result.error;
        if (result.status !== 0) {
            const error = new Error(
                result.stderr.trim() || `FusionDTL exited with ${result.status}`,
            );
            error.exitCode = result.status;
            error.stdout = result.stdout;
            error.stderr = result.stderr;
            throw error;
        }
        return normalizeReport(JSON.parse(result.stdout));
    }

    snapshot(name = "settle") {
        return operationalSnapshot(this.run(name));
    }
}

module.exports = { FusionClient };

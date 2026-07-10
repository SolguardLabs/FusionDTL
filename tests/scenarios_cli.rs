use std::process::Command;

use serde_json::Value;

fn run_scenario(args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_fusion_dtl"))
        .args(args)
        .output()
        .expect("the fusion_dtl binary should execute");

    assert!(
        output.status.success(),
        "scenario failed\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("scenario output should be valid JSON")
}

fn field<'a>(value: &'a Value, path: &[&str]) -> &'a Value {
    let mut cursor = value;
    for segment in path {
        cursor = cursor
            .get(*segment)
            .unwrap_or_else(|| panic!("missing JSON field: {}", path.join(".")));
    }
    cursor
}

fn text(value: &Value, path: &[&str]) -> String {
    field(value, path)
        .as_str()
        .unwrap_or_else(|| panic!("field should be a string: {}", path.join(".")))
        .to_owned()
}

fn number(value: &Value, path: &[&str]) -> u64 {
    field(value, path)
        .as_u64()
        .unwrap_or_else(|| panic!("field should be a number: {}", path.join(".")))
}

fn boolean(value: &Value, path: &[&str]) -> bool {
    field(value, path)
        .as_bool()
        .unwrap_or_else(|| panic!("field should be a boolean: {}", path.join(".")))
}

#[test]
fn snapshot_reports_bootstrapped_surface() {
    let report = run_scenario(&["snapshot"]);

    assert_eq!(text(&report, &["scenario"]), "snapshot");
    assert!(field(&report, &["receipt"]).is_null());
    assert_eq!(number(&report, &["surface", "receipts"]), 0);
    assert_eq!(number(&report, &["surface", "processed_packets"]), 0);
    assert_eq!(number(&report, &["surface", "participant_profiles"]), 6);
    assert_eq!(number(&report, &["surface", "active_profiles"]), 6);
    assert_eq!(number(&report, &["surface", "settlement_windows"]), 1);
    assert_eq!(number(&report, &["surface", "operators"]), 5);
    assert_eq!(number(&report, &["surface", "role_assignments"]), 7);
    assert_eq!(number(&report, &["surface", "delivery_lanes"]), 2);
    assert_eq!(number(&report, &["surface", "relayer_quotes"]), 2);
    assert_eq!(number(&report, &["surface", "capacity_policies"]), 2);
    assert_eq!(number(&report, &["journal_entries"]), 28);
    assert!(boolean(&report, &["conservation_ok"]));
}

#[test]
fn issue_moves_value_into_edge_liability() {
    let report = run_scenario(&["issue"]);

    assert_eq!(text(&report, &["scenario"]), "issue");
    assert_eq!(number(&report, &["receipt", "amount"]), 2_500_000_000);
    assert_eq!(number(&report, &["balances", "issuer"]), 17_500_000_000);
    assert_eq!(number(&report, &["cells", "edge_reserve"]), 2_500_000_000);
    assert_eq!(number(&report, &["cells", "edge_pending"]), 2_500_000_000);
    assert_eq!(number(&report, &["surface", "receipts"]), 1);
    assert_eq!(number(&report, &["surface", "exposure_cells"]), 1);
    assert!(boolean(&report, &["conservation_ok"]));
}

#[test]
fn settle_distributes_amounts_and_closes_liability() {
    let report = run_scenario(&["settle"]);

    assert_eq!(text(&report, &["scenario"]), "settle");
    assert_eq!(number(&report, &["balances", "beneficiary"]), 2_495_000_000);
    assert_eq!(number(&report, &["balances", "relayer"]), 5_000_000);
    assert_eq!(number(&report, &["cells", "edge_reserve"]), 0);
    assert_eq!(number(&report, &["cells", "edge_pending"]), 0);
    assert_eq!(number(&report, &["surface", "processed_packets"]), 1);
    assert_eq!(number(&report, &["surface", "treasury_assets"]), 1);
    assert_eq!(
        field(&report, &["transactions"])
            .as_array()
            .expect("transactions should be an array")
            .len(),
        2
    );
    assert!(boolean(&report, &["conservation_ok"]));
}

#[test]
fn rebalance_settles_through_core_cell_and_reports_exposure() {
    let report = run_scenario(&["rebalance"]);

    assert_eq!(text(&report, &["scenario"]), "rebalance");
    assert_eq!(
        number(&report, &["balances", "edge_controller"]),
        2_500_000_000
    );
    assert_eq!(number(&report, &["cells", "core_reserve"]), 87_500_000_000);
    assert_eq!(number(&report, &["cells", "edge_reserve"]), 0);
    assert_eq!(number(&report, &["cells", "edge_pending"]), 0);
    assert_eq!(number(&report, &["surface", "exposure_cells"]), 2);
    assert_eq!(
        field(&report, &["transactions"])
            .as_array()
            .expect("transactions should be an array")
            .len(),
        3
    );
    assert!(boolean(&report, &["conservation_ok"]));
}

#[test]
fn default_and_unknown_scenarios_resolve_to_settle() {
    let default_report = run_scenario(&[]);
    let unknown_report = run_scenario(&["unknown"]);

    assert_eq!(text(&default_report, &["scenario"]), "settle");
    assert_eq!(text(&unknown_report, &["scenario"]), "settle");
    assert_eq!(
        number(&default_report, &["surface", "processed_packets"]),
        number(&unknown_report, &["surface", "processed_packets"])
    );
    assert!(boolean(&default_report, &["conservation_ok"]));
    assert!(boolean(&unknown_report, &["conservation_ok"]));
}

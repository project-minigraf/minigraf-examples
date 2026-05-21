use minigraf_examples::scenarios;

#[test]
fn agentic_memory_records_and_queries_agent_context() {
    let lines = scenarios::agentic_memory().expect("agentic memory scenario runs");

    assert_eq!(
        lines,
        [
            "Agent memory: remembered Alice prefers concise technical answers.",
            "Agent memory: retrieved Alice's current project, minigraf-examples.",
            "Agent memory: wrote a correction with transaction history intact."
        ]
    );
}

#[test]
fn offline_first_mobile_records_local_changes_before_sync() {
    let lines = scenarios::offline_first_mobile().expect("offline-first mobile scenario runs");

    assert_eq!(
        lines,
        [
            "Offline mobile: stored two local task changes while disconnected.",
            "Offline mobile: selected the pending changes for later sync.",
            "Offline mobile: marked the synced task without losing local history."
        ]
    );
}

#[test]
fn audit_log_tracks_historical_decisions() {
    let lines = scenarios::audit_log().expect("audit log scenario runs");

    assert_eq!(
        lines,
        [
            "Audit log: recorded policy approval and superseding revision.",
            "Audit log: queried the current policy owner.",
            "Audit log: queried transaction-time history for the earlier owner."
        ]
    );
}

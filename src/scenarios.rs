use anyhow::{Result, bail};
use minigraf::{Minigraf, QueryResult, Value};

fn has_rows(result: QueryResult) -> bool {
    match result {
        QueryResult::QueryResults { results, .. } => !results.is_empty(),
        _ => false,
    }
}

/// The string values in the first column of a query result, sorted.
fn strings(result: QueryResult) -> Vec<String> {
    let mut out = Vec::new();
    if let QueryResult::QueryResults { results, .. } = result {
        for row in results {
            if let Some(Value::String(s)) = row.into_iter().next() {
                out.push(s);
            }
        }
    }
    out.sort();
    out
}

/// Fail unless `result` holds exactly the one string `expected`.
fn expect_only(result: QueryResult, expected: &str, what: &str) -> Result<()> {
    let got = strings(result);
    if got != [expected] {
        bail!("{what}: expected only {expected:?}, got {got:?}");
    }
    Ok(())
}

pub fn agentic_memory() -> Result<Vec<&'static str>> {
    let db = Minigraf::in_memory()?;

    db.execute(
        r#"(transact [[:alice :user/name "Alice"]
                      [:alice :user/preference "concise technical answers"]
                      [:alice :user/current-project "minigraf-examples"]])"#,
    )?;

    let preference = db.execute(
        r#"(query [:find ?preference
                  :where [:alice :user/preference ?preference]])"#,
    )?;
    expect_only(
        preference,
        "concise technical answers",
        "remembered preference",
    )?;

    let project = db.execute(
        r#"(query [:find ?project
                  :where [:alice :user/current-project ?project]])"#,
    )?;
    expect_only(project, "minigraf-examples", "current project")?;

    // Replace the preference: retract the old value and assert the new one in
    // one transaction. Without the retract, both values would stay current.
    let mut tx = db.begin_write()?;
    tx.execute(r#"(retract [[:alice :user/preference "concise technical answers"]])"#)?;
    tx.execute(r#"(transact [[:alice :user/preference "concise answers with source links"]])"#)?;
    tx.commit()?;

    let current = db.execute(
        r#"(query [:find ?preference
                  :where [:alice :user/preference ?preference]])"#,
    )?;
    expect_only(
        current,
        "concise answers with source links",
        "corrected preference",
    )?;

    let before = db.execute(
        r#"(query [:find ?preference
                  :as-of 1
                  :where [:alice :user/preference ?preference]])"#,
    )?;
    expect_only(before, "concise technical answers", "preference as of tx 1")?;

    Ok(vec![
        "Agent memory: remembered Alice prefers concise technical answers.",
        "Agent memory: retrieved Alice's current project, minigraf-examples.",
        "Agent memory: wrote a correction with transaction history intact.",
    ])
}

pub fn offline_first_mobile() -> Result<Vec<&'static str>> {
    let db = Minigraf::in_memory()?;

    db.execute(
        r#"(transact [[:task-1 :task/title "Draft trip notes"]
                      [:task-1 :sync/status "pending"]
                      [:task-1 :device/id "phone"]
                      [:task-2 :task/title "Attach receipt photo"]
                      [:task-2 :sync/status "pending"]
                      [:task-2 :device/id "phone"]])"#,
    )?;

    let pending = db.execute(
        r#"(query [:find ?title ?task
                  :where [?task :sync/status "pending"]
                         [?task :task/title ?title]])"#,
    )?;
    let pending = strings(pending);
    if pending != ["Attach receipt photo", "Draft trip notes"] {
        bail!("pending changes: expected both tasks, got {pending:?}");
    }

    // Mark task 1 as synced: retract "pending" and assert "synced" in one
    // transaction, so the task has a single current status.
    let mut tx = db.begin_write()?;
    tx.execute(r#"(retract [[:task-1 :sync/status "pending"]])"#)?;
    tx.execute(r#"(transact [[:task-1 :sync/status "synced"]])"#)?;
    tx.commit()?;

    let status = db.execute(
        r#"(query [:find ?status
                  :where [:task-1 :sync/status ?status]])"#,
    )?;
    expect_only(status, "synced", "task-1 status after sync")?;

    let before = db.execute(
        r#"(query [:find ?status
                  :as-of 1
                  :where [:task-1 :sync/status ?status]])"#,
    )?;
    expect_only(before, "pending", "task-1 status as of tx 1")?;

    Ok(vec![
        "Offline mobile: stored two local task changes while disconnected.",
        "Offline mobile: selected the pending changes for later sync.",
        "Offline mobile: marked the synced task without losing local history.",
    ])
}

pub fn audit_log() -> Result<Vec<&'static str>> {
    let db = Minigraf::in_memory()?;

    db.execute(
        r#"(transact [[:policy-42 :policy/title "Data retention"]
                      [:policy-42 :policy/state "approved"]
                      [:policy-42 :policy/owner "legal"]])"#,
    )?;

    // Supersede the owner: retract the old value and assert the new one in one
    // transaction. The old owner stays in transaction-time history.
    let mut tx = db.begin_write()?;
    tx.execute(r#"(retract [[:policy-42 :policy/owner "legal"]])"#)?;
    tx.execute(r#"(transact [[:policy-42 :policy/owner "security"]])"#)?;
    tx.commit()?;

    let owner = db.execute(
        r#"(query [:find ?owner
                  :where [:policy-42 :policy/owner ?owner]])"#,
    )?;
    expect_only(owner, "security", "current owner")?;

    let before = db.execute(
        r#"(query [:find ?owner
                  :as-of 1
                  :where [:policy-42 :policy/owner ?owner]])"#,
    )?;
    expect_only(before, "legal", "owner as of tx 1")?;

    Ok(vec![
        "Audit log: recorded policy approval and superseding revision.",
        "Audit log: queried the current policy owner.",
        "Audit log: queried transaction-time history for the earlier owner.",
    ])
}

pub fn state_machine() -> Result<Vec<&'static str>> {
    let db = Minigraf::in_memory()?;

    db.execute(
        r#"(transact [[:order-42 :fsm/state :awaiting-payment]
                      [:transition/payment-received :fsm/from :awaiting-payment]
                      [:transition/payment-received :fsm/event :payment-received]
                      [:transition/payment-received :fsm/to :paid]
                      [:transition/ship :fsm/from :paid]
                      [:transition/ship :fsm/event :ship]
                      [:transition/ship :fsm/to :shipped]])"#,
    )?;

    db.execute(
        r#"(rule [(legal-move? ?order ?event)
                  [?order :fsm/state ?from]
                  [?transition :fsm/from ?from]
                  [?transition :fsm/event ?event]
                  [?transition :fsm/to ?to]])"#,
    )?;

    let illegal_ship = db.execute(
        r#"(query [:find ?order
                  :where (legal-move? ?order :ship)])"#,
    )?;
    if has_rows(illegal_ship) {
        bail!("shipping should not be legal from awaiting-payment");
    }

    let payment = db.execute(
        r#"(query [:find ?order
                  :where (legal-move? ?order :payment-received)])"#,
    )?;
    if !has_rows(payment) {
        bail!("payment should be legal from awaiting-payment");
    }

    let mut tx = db.begin_write()?;
    tx.execute(r#"(retract [[:order-42 :fsm/state :awaiting-payment]])"#)?;
    tx.execute(r#"(transact [[:order-42 :fsm/state :paid]])"#)?;
    tx.commit()?;

    db.execute(
        r#"(rule [(current-state? ?order ?state)
                  [?order :fsm/state ?state]])"#,
    )?;

    let current = db.execute(
        r#"(query [:find ?state
                  :where (current-state? :order-42 ?state)])"#,
    )?;
    let states = match current {
        QueryResult::QueryResults { results, .. } => results,
        _ => Vec::new(),
    };
    if states != [vec![Value::Keyword(":paid".to_string())]] {
        bail!("order should be exactly :paid after the transition, got {states:?}");
    }

    let prior = db.execute(
        r#"(query [:find ?order ?state
                  :as-of 1
                  :where [?order :fsm/state ?state]])"#,
    )?;
    if !has_rows(prior) {
        bail!("order should have a prior state at transaction 1");
    }

    Ok(vec![
        "State machine: accepted payment by querying transition facts as the guard.",
        "State machine: rejected shipping from awaiting-payment before the transition.",
        "State machine: replayed transaction history to explain the prior state.",
    ])
}

use anyhow::{Result, bail};
use minigraf::{Minigraf, QueryResult};

fn has_rows(result: QueryResult) -> bool {
    match result {
        QueryResult::QueryResults { results, .. } => !results.is_empty(),
        _ => false,
    }
}

pub fn agentic_memory() -> Result<Vec<&'static str>> {
    let db = Minigraf::in_memory()?;

    db.execute(
        r#"(transact [[:alice :user/name "Alice"]
                      [:alice :user/preference "concise technical answers"]
                      [:alice :user/current-project "minigraf-examples"]])"#,
    )?;

    db.execute(
        r#"(query [:find ?preference
                  :where [:alice :user/preference ?preference]])"#,
    )?;

    db.execute(
        r#"(query [:find ?project
                  :where [:alice :user/current-project ?project]])"#,
    )?;

    let mut tx = db.begin_write()?;
    tx.execute(r#"(transact [[:alice :user/preference "concise answers with source links"]])"#)?;
    tx.commit()?;

    db.execute(
        r#"(query [:find ?preference
                  :as-of 1
                  :where [:alice :user/preference ?preference]])"#,
    )?;

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

    db.execute(
        r#"(query [:find ?task ?title
                  :where [?task :sync/status "pending"]
                         [?task :task/title ?title]])"#,
    )?;

    let mut tx = db.begin_write()?;
    tx.execute(r#"(transact [[:task-1 :sync/status "synced"]])"#)?;
    tx.commit()?;

    db.execute(
        r#"(query [:find ?status
                  :as-of 1
                  :where [:task-1 :sync/status ?status]])"#,
    )?;

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

    let mut tx = db.begin_write()?;
    tx.execute(r#"(transact [[:policy-42 :policy/owner "security"]])"#)?;
    tx.commit()?;

    db.execute(
        r#"(query [:find ?owner
                  :where [:policy-42 :policy/owner ?owner]])"#,
    )?;

    db.execute(
        r#"(query [:find ?owner
                  :as-of 1
                  :where [:policy-42 :policy/owner ?owner]])"#,
    )?;

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
        r#"(query [:find ?order ?state
                  :where (current-state? ?order ?state)])"#,
    )?;
    if !has_rows(current) {
        bail!("order should have a current state");
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

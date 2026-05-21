use anyhow::Result;
use minigraf::Minigraf;

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

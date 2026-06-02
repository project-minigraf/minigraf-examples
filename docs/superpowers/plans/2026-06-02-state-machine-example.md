# State Machine Example Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a runnable state machine example demonstrating Minigraf as a durable, auditable guard-evaluation substrate.

**Architecture:** Follow the existing example pattern: `src/scenarios.rs` owns executable scenario logic; `examples/state_machine.rs` prints the scenario lines; `tests/scenarios.rs` pins exact output; `README.md` documents how to run it. The guard is a Datalog query over transition facts and current state, and the state update is an explicit write transaction containing one `retract` and one `transact`.

**Tech Stack:** Rust 2024, `anyhow`, `minigraf = "1.1"`, Cargo integration tests.

---

### Task 1: Pin the scenario output

**Files:**
- Modify: `tests/scenarios.rs`

- [ ] **Step 1: Write the failing test**

Add:

```rust
#[test]
fn state_machine_guards_transitions_and_replays_history() {
    let lines = scenarios::state_machine().expect("state machine scenario runs");

    assert_eq!(
        lines,
        [
            "State machine: accepted payment by querying transition facts as the guard.",
            "State machine: rejected shipping from awaiting-payment before the transition.",
            "State machine: replayed transaction history to explain the prior state."
        ]
    );
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test state_machine_guards_transitions_and_replays_history`

Expected: compile failure because `scenarios::state_machine` does not exist yet.

### Task 2: Implement scenario logic

**Files:**
- Modify: `src/scenarios.rs`

- [ ] **Step 1: Add a helper to detect non-empty query results**

```rust
fn has_rows(result: minigraf::QueryResult) -> bool {
    match result {
        minigraf::QueryResult::QueryResults { results, .. } => !results.is_empty(),
        _ => false,
    }
}
```

- [ ] **Step 2: Add `state_machine()`**

Implement initialization facts, guard queries, explicit write transaction with `retract` plus `transact`, current-state query, and `:as-of 1` query.

- [ ] **Step 3: Run the test to verify it passes**

Run: `cargo test state_machine_guards_transitions_and_replays_history`

Expected: pass.

### Task 3: Add the runnable example and docs

**Files:**
- Create: `examples/state_machine.rs`
- Modify: `README.md`

- [ ] **Step 1: Add the binary**

```rust
use anyhow::Result;

fn main() -> Result<()> {
    for line in minigraf_examples::scenarios::state_machine()? {
        println!("{line}");
    }

    Ok(())
}
```

- [ ] **Step 2: Add README section**

Document the scenario, run command, and expected three-line output.

- [ ] **Step 3: Verify the full crate**

Run: `cargo test`

Expected: all tests pass.

Run: `cargo run --example state_machine`

Expected: prints the three documented lines.

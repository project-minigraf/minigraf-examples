# minigraf-examples

Examples, integrations, and cookbooks for Minigraf -- part of the Minigraf ecosystem.

This repository is a standalone Rust examples crate. Each scenario is runnable with
Cargo and uses the published [`minigraf`](https://crates.io/crates/minigraf) crate.

## Prerequisites

- Rust 1.85 or newer for edition 2024 crates.
- Cargo with access to crates.io.
- `minigraf = "1.1"` from crates.io. Cargo currently resolves this to
  `minigraf v1.1.1`.

Install and verify dependencies:

```sh
cargo test
```

## Scenarios

### Agentic Memory

Stores user preferences and project context in Minigraf, queries them back for an
agent response, then writes a correction while keeping transaction-time history.

Run:

```sh
cargo run --example agentic_memory
```

Expected output:

```text
Agent memory: remembered Alice prefers concise technical answers.
Agent memory: retrieved Alice's current project, minigraf-examples.
Agent memory: wrote a correction with transaction history intact.
```

### Offline-First Mobile

Stores local task changes while disconnected, queries pending changes for a future
sync pass, then records a synced state while retaining earlier transaction state.

Run:

```sh
cargo run --example offline_first_mobile
```

Expected output:

```text
Offline mobile: stored two local task changes while disconnected.
Offline mobile: selected the pending changes for later sync.
Offline mobile: marked the synced task without losing local history.
```

### Audit Log

Records a policy approval, supersedes the owner, then queries both current state
and an earlier transaction-time view.

Run:

```sh
cargo run --example audit_log
```

Expected output:

```text
Audit log: recorded policy approval and superseding revision.
Audit log: queried the current policy owner.
Audit log: queried transaction-time history for the earlier owner.
```

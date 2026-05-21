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

## LangChain Integrations

The LangChain examples use the published language bindings directly:

- Python: `minigraf==1.1.1`
- Node.js: `minigraf@1.1.1`

### Python LangChain

Implements `BaseChatMessageHistory` from `langchain-core` with Minigraf-backed
message storage.

Install prerequisites:

```sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r integrations/langchain-python/requirements.txt
```

Run:

```sh
python integrations/langchain-python/minigraf_chat_history.py
```

Expected output:

```text
Human: Remember that Minigraf stores agent memory.
AI: Got it. I will use Minigraf-backed chat history.
```

### LangChain.js

Implements `BaseChatMessageHistory` from `@langchain/core/chat_history` with
Minigraf-backed message storage.

Install prerequisites:

```sh
cd integrations/langchain-js
npm install
```

Run:

```sh
npm start
```

Expected output:

```text
Human: Remember that Minigraf stores agent memory.
AI: Got it. I will use Minigraf-backed chat history.
```

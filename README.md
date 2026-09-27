# minigraf-examples

Examples, integrations, and cookbooks for Minigraf -- part of the Minigraf ecosystem.

This repository is a standalone Rust examples crate. Each scenario is runnable with
Cargo and uses the published [`minigraf`](https://crates.io/crates/minigraf) crate.

Most scenarios below also have a link to the
[Minigraf time travel visualizer](https://project-minigraf.github.io/minigraf-visualizer/).
The link replays the scenario's writes in your browser, so you can step through its
transactions and see the facts over transaction time and valid time. The browser engine
has no multi-statement write transactions, so where a scenario retracts an old value and
asserts the new one in one atomic transaction, the visualizer shows two transactions.

## Prerequisites

- Rust 1.89 or newer (the minimum supported Rust version of `minigraf` 2.x).
- Cargo with access to crates.io.
- `minigraf = "2.0"` from crates.io. Cargo currently resolves this to
  `minigraf v2.0.2`.

Install and verify dependencies:

```sh
cargo test
```

## Scenarios

### Agentic Memory

Stores user preferences and project context in Minigraf, queries them back for an
agent response, then corrects the preference: one write transaction retracts the old
value and asserts the new one. The old value stays in transaction-time history.

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

[See this scenario in the time travel visualizer →](https://project-minigraf.github.io/minigraf-visualizer/#data=KHRyYW5zYWN0IFtbOmFsaWNlIDp1c2VyL25hbWUgIkFsaWNlIl0KICBbOmFsaWNlIDp1c2VyL3ByZWZlcmVuY2UgImNvbmNpc2UgdGVjaG5pY2FsIGFuc3dlcnMiXQogIFs6YWxpY2UgOnVzZXIvY3VycmVudC1wcm9qZWN0ICJtaW5pZ3JhZi1leGFtcGxlcyJdXSkKKHJldHJhY3QgW1s6YWxpY2UgOnVzZXIvcHJlZmVyZW5jZSAiY29uY2lzZSB0ZWNobmljYWwgYW5zd2VycyJdXSkKKHRyYW5zYWN0IFtbOmFsaWNlIDp1c2VyL3ByZWZlcmVuY2UgImNvbmNpc2UgYW5zd2VycyB3aXRoIHNvdXJjZSBsaW5rcyJdXSk&title=minigraf-examples:+agentic+memory&vt=any&e=:alice&view=map)
Step back to tx 1 to see what the agent remembered before the update.

### Offline-First Mobile

Stores local task changes while disconnected, queries pending changes for a future
sync pass, then marks a task synced by retracting `"pending"` and asserting `"synced"`
in one write transaction, while retaining earlier transaction state.

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

[See this scenario in the time travel visualizer →](https://project-minigraf.github.io/minigraf-visualizer/#data=KHRyYW5zYWN0IFtbOnRhc2stMSA6dGFzay90aXRsZSAiRHJhZnQgdHJpcCBub3RlcyJdCiAgWzp0YXNrLTEgOnN5bmMvc3RhdHVzICJwZW5kaW5nIl0KICBbOnRhc2stMSA6ZGV2aWNlL2lkICJwaG9uZSJdCiAgWzp0YXNrLTIgOnRhc2svdGl0bGUgIkF0dGFjaCByZWNlaXB0IHBob3RvIl0KICBbOnRhc2stMiA6c3luYy9zdGF0dXMgInBlbmRpbmciXQogIFs6dGFzay0yIDpkZXZpY2UvaWQgInBob25lIl1dKQoocmV0cmFjdCBbWzp0YXNrLTEgOnN5bmMvc3RhdHVzICJwZW5kaW5nIl1dKQoodHJhbnNhY3QgW1s6dGFzay0xIDpzeW5jL3N0YXR1cyAic3luY2VkIl1dKQ&title=minigraf-examples:+offline-first+mobile&vt=any&e=:task-1&view=map)

### State Machine

Initializes an order in `:awaiting-payment` and stores legal transitions as Minigraf facts.
A Datalog guard query checks whether an event is legal from the order's current state.
A legal payment transition is accepted; an illegal ship event is rejected. The state
update is an atomic write transaction containing a `retract` and a `transact`. A
transaction-time replay with `:as-of 1` shows the prior state.

Run:

```sh
cargo run --example state_machine
```

Expected output:

```text
State machine: accepted payment by querying transition facts as the guard.
State machine: rejected shipping from awaiting-payment before the transition.
State machine: replayed transaction history to explain the prior state.
```

[See this scenario in the time travel visualizer →](https://project-minigraf.github.io/minigraf-visualizer/#data=KHRyYW5zYWN0IFtbOm9yZGVyLTQyIDpmc20vc3RhdGUgOmF3YWl0aW5nLXBheW1lbnRdCiAgWzp0cmFuc2l0aW9uL3BheW1lbnQtcmVjZWl2ZWQgOmZzbS9mcm9tIDphd2FpdGluZy1wYXltZW50XQogIFs6dHJhbnNpdGlvbi9wYXltZW50LXJlY2VpdmVkIDpmc20vZXZlbnQgOnBheW1lbnQtcmVjZWl2ZWRdCiAgWzp0cmFuc2l0aW9uL3BheW1lbnQtcmVjZWl2ZWQgOmZzbS90byA6cGFpZF0KICBbOnRyYW5zaXRpb24vc2hpcCA6ZnNtL2Zyb20gOnBhaWRdCiAgWzp0cmFuc2l0aW9uL3NoaXAgOmZzbS9ldmVudCA6c2hpcF0KICBbOnRyYW5zaXRpb24vc2hpcCA6ZnNtL3RvIDpzaGlwcGVkXV0pCihydWxlIFsobGVnYWwtbW92ZT8gP29yZGVyID9ldmVudCkKICBbP29yZGVyIDpmc20vc3RhdGUgP2Zyb21dCiAgWz90cmFuc2l0aW9uIDpmc20vZnJvbSA_ZnJvbV0KICBbP3RyYW5zaXRpb24gOmZzbS9ldmVudCA_ZXZlbnRdCiAgWz90cmFuc2l0aW9uIDpmc20vdG8gP3RvXV0pCihyZXRyYWN0IFtbOm9yZGVyLTQyIDpmc20vc3RhdGUgOmF3YWl0aW5nLXBheW1lbnRdXSkKKHRyYW5zYWN0IFtbOm9yZGVyLTQyIDpmc20vc3RhdGUgOnBhaWRdXSkKKHJ1bGUgWyhjdXJyZW50LXN0YXRlPyA_b3JkZXIgP3N0YXRlKQogIFs_b3JkZXIgOmZzbS9zdGF0ZSA_c3RhdGVdXSk&title=minigraf-examples:+state+machine&tx=1&e=:order-42)
Press → to watch the order move from `:awaiting-payment` to `:paid`: the retract (tx 2),
then the transact (tx 3).

### Audit Log

Records a policy approval, supersedes the owner (retract the old owner and assert
the new one in one write transaction), then queries both current state and an
earlier transaction-time view.

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

[See this scenario in the time travel visualizer →](https://project-minigraf.github.io/minigraf-visualizer/#data=KHRyYW5zYWN0IFtbOnBvbGljeS00MiA6cG9saWN5L3RpdGxlICJEYXRhIHJldGVudGlvbiJdCiAgWzpwb2xpY3ktNDIgOnBvbGljeS9zdGF0ZSAiYXBwcm92ZWQiXQogIFs6cG9saWN5LTQyIDpwb2xpY3kvb3duZXIgImxlZ2FsIl1dKQoocmV0cmFjdCBbWzpwb2xpY3ktNDIgOnBvbGljeS9vd25lciAibGVnYWwiXV0pCih0cmFuc2FjdCBbWzpwb2xpY3ktNDIgOnBvbGljeS9vd25lciAic2VjdXJpdHkiXV0p&title=minigraf-examples:+audit+log&vt=any&e=:policy-42&view=map)

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

## Ecosystem Crates

### minigraf-algorithms

`minigraf-algorithms/` is a standalone crate for graph algorithms that operate
on Minigraf data. It lives outside Minigraf core so traversal helpers can evolve
as opt-in ecosystem utilities without enlarging the embedded database API.
It includes breadth-first and depth-first traversal, shortest path, topological
sort, and connected components.

Run:

```sh
cargo test -p minigraf-algorithms
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

## GraphRAG Integration

The GraphRAG example uses ChromaDB for semantic retrieval and Minigraf for structured
graph traversal. Entity UUIDs are the explicit bridge between the two stores.

- Python: `minigraf==1.1.1`, `chromadb`

### Python GraphRAG

Populates a six-node concept graph in Minigraf, indexes entity descriptions in an
in-memory ChromaDB collection, retrieves the closest entity by semantic similarity,
then traverses its graph neighbours.

Install prerequisites:

```sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r integrations/graphrag-python/requirements.txt
```

Note: ChromaDB downloads the `all-MiniLM-L6-v2` embedding model (~80 MB) on first run.

Run:

```sh
python integrations/graphrag-python/graphrag_minigraf.py
```

Expected output:

```text
Query: "storing time-varying relationships"
Match: temporal-data
Related: graph-db, knowledge-graph
```

## LlamaIndex Integration

`MinigrafGraphStore` implements LlamaIndex's `SimpleGraphStore` with Minigraf as the
backing store. Triplets map directly onto Minigraf's native entity-attribute-value model:
subjects and predicates become Minigraf keywords (`:myapp`, `:depends-on`), objects are
string values. The example shows tech-stack dependency evolution: a v1 graph is ingested,
a dependency is upgraded, the current state is queried via `get_rel_map`, and Minigraf's
`:as-of <tx>` temporal query recovers the pre-upgrade state from transaction history.

- Python: `minigraf==1.1.1`, `llama-index-core`

### Python LlamaIndex

Subclasses `SimpleGraphStore` from `llama_index.core.graph_stores` with Minigraf-backed
triplet storage and a raw Datalog pass-through in `query()`.

Install prerequisites:

```sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r integrations/llamaindex-python/requirements.txt
```

Run:

```sh
python integrations/llamaindex-python/minigraf_graph_store.py
```

Expected output:

```text
myapp depends-on: pydantic==2.0, requests==2.28
requests depends-on: urllib3==1.26
myapp at tx 3 depended-on: pydantic==1.10, requests==2.28
```

For an interactive version of the same idea, open the
[dependency upgrades sample](https://project-minigraf.github.io/minigraf-visualizer/#sample=dependencies&tx=5)
in the time travel visualizer and step through the releases.

**Temporal model:** Each `upsert_triplet` call is one Minigraf transaction. After three
ingestion calls, `SNAPSHOT_TX = 3`. Retracting and re-asserting a dependency advances the
transaction counter. `(query [:find ?o :as-of 3 :where [:myapp :depends-on ?o]])` returns
the dependency set as it existed after transaction 3 — before the upgrade.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

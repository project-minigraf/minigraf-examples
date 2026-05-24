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

## Ecosystem Crates

### minigraf-algorithms

`minigraf-algorithms/` is a standalone crate for graph algorithms that operate
on Minigraf data. It lives outside Minigraf core so traversal helpers can evolve
as opt-in ecosystem utilities without enlarging the embedded database API.

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

**Temporal model:** Each `upsert_triplet` call is one Minigraf transaction. After three
ingestion calls, `SNAPSHOT_TX = 3`. Retracting and re-asserting a dependency advances the
transaction counter. `(query [:find ?o :as-of 3 :where [:myapp :depends-on ?o]])` returns
the dependency set as it existed after transaction 3 — before the upgrade.

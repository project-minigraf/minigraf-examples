# LlamaIndex Integration Design

**Date:** 2026-05-24  
**Issue:** #5 — post-1.0: add LlamaIndex integration example

---

## Overview

Add a `MinigrafGraphStore` that implements LlamaIndex's `SimpleGraphStore` abstract class, with Minigraf as the backing store. The example demonstrates tech-stack dependency evolution to showcase Minigraf's temporal graph model.

---

## File Layout

```
integrations/llamaindex-python/
  minigraf_graph_store.py   # MinigrafGraphStore class + main()
  requirements.txt           # minigraf==1.1.1, llama-index-core
```

Static tests are added to `tests/langchain_integrations.py` (existing test file for Python integration smoke tests).

---

## Data Model

Triplets map directly onto Minigraf's native entity-attribute-value (EAV) model:

- **Subject** → Minigraf keyword entity (e.g. `"myapp"` → `:myapp`)
- **Predicate** → Minigraf attribute keyword (e.g. `"depends-on"` → `:depends-on`)
- **Object** → JSON-quoted string value (e.g. `"pydantic==1.10"`)

String normalisation: lowercase, non-alphanumeric characters replaced with `-`, prefixed with `:`.

### Cardinality behaviour (verified against Minigraf 1.1.1)

- Each `execute("(transact [[...]])")` call is additive. Multiple values for the same `(entity, attribute)` pair coexist when asserted in separate transactions.
- Transacting an already-existing `(entity, attribute, value)` triple is idempotent — no duplicate is stored.
- Within a single `transact` list, asserting the same `(entity, attribute)` twice keeps only the last value.
- `retract` explicitly removes a specific `(entity, attribute, value)` triple.

Consequence: `upsert_triplet` is a plain `transact` with no prior retract needed. A dependency upgrade requires an explicit `delete_triplet` + `upsert_triplet`.

### Temporal queries

`:as-of` is placed inside the Datalog query string and takes a 1-based transaction ordinal:

```
(query [:find ?o :as-of 3 :where [:myapp :depends-on ?o]])
```

The transaction counter increments once per `execute("(transact ...)")` or `execute("(retract ...)")` call.

---

## GraphStore Interface

| Method | Minigraf operation |
|---|---|
| `upsert_triplet(subj, rel, obj)` | `(transact [[:<subj> :<rel> "<obj>"]])` |
| `delete_triplet(subj, rel, obj)` | `(retract [[:<subj> :<rel> "<obj>"]])` |
| `get(subj)` | `(query [:find ?r ?o :where [:<subj> ?r ?o]])` |
| `get_rel_map(subjs, depth, limit)` | one `get()` per subject, merged into `{subj: [[rel, obj], ...]}` |
| `query(query, param_map)` | `db.execute(query)` — raw Datalog pass-through |

`get_rel_map` uses one query per subject because Minigraf's `or` clause does not preserve subject identity in the result set (no `ground` or `str` predicates available). The `depth` and `limit` parameters are accepted but ignored in this example — depth-1 traversal is sufficient to tell the story.

---

## Scenario Flow

The `main()` function demonstrates a four-beat narrative. The graph represents a Python tech stack.

### Beat 1 — Ingest v1 (transactions 1–3)

```python
store.upsert_triplet("myapp",    "depends-on", "pydantic==1.10")
store.upsert_triplet("myapp",    "depends-on", "requests==2.28")
store.upsert_triplet("requests", "depends-on", "urllib3==1.26")
# snapshot_tx = 3
```

### Beat 2 — Upgrade pydantic (transactions 4–5)

```python
store.delete_triplet("myapp", "depends-on", "pydantic==1.10")
store.upsert_triplet("myapp", "depends-on", "pydantic==2.0")
```

### Beat 3 — Query current state

```python
store.get_rel_map(["myapp", "requests"])
# → myapp:    [["depends-on", "pydantic==2.0"], ["depends-on", "requests==2.28"]]
# → requests: [["depends-on", "urllib3==1.26"]]
```

### Beat 4 — Query historical state

```python
db.execute(f"(query [:find ?o :as-of {SNAPSHOT_TX} :where [:myapp :depends-on ?o]])")
# → pydantic==1.10, requests==2.28
```

`SNAPSHOT_TX = 3` is a module-level constant (deterministic given the fixed operation sequence).

---

## EXPECTED_OUTPUT

```
myapp depends-on: pydantic==2.0, requests==2.28
requests depends-on: urllib3==1.26
myapp at tx 3 depended-on: pydantic==1.10, requests==2.28
```

---

## Tests

Two tests added to `tests/langchain_integrations.py`:

- `test_llamaindex_integration_pins_minigraf_1_1_1` — asserts `minigraf==1.1.1` and `llama-index-core` in `requirements.txt`
- `test_llamaindex_example_documents_expected_output` — asserts `class MinigrafGraphStore` and the three `EXPECTED_OUTPUT` lines are present in the source file

---

## README

New **LlamaIndex Integration** section added after the GraphRAG section, including:

- One-paragraph description of what the example shows
- Install and run instructions
- Expected output block
- Note mapping Minigraf's temporal model: direct EAV storage of triplets, `:as-of <tx>` for transaction-time history

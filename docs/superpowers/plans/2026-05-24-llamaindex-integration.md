# LlamaIndex Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `MinigrafGraphStore` that implements LlamaIndex's `SimpleGraphStore`, backed by Minigraf, with a runnable tech-stack dependency-evolution example that showcases Minigraf's temporal query model.

**Architecture:** `MinigrafGraphStore` subclasses `SimpleGraphStore` from `llama_index.core.graph_stores` and stores triplets as native Minigraf EAV facts (`[:subject :predicate "object"]`). The `main()` function ingests a v1 dependency graph, upgrades a dependency, queries current state via `get_rel_map`, then queries pre-upgrade history using Minigraf's `:as-of` transaction ordinal.

**Tech Stack:** Python 3, `minigraf==1.1.1` (PyPI), `llama-index-core` (PyPI), `pytest` (for static tests).

---

## File Map

| Action | Path | Responsibility |
|---|---|---|
| Create | `integrations/llamaindex-python/requirements.txt` | Pin dependencies |
| Create | `integrations/llamaindex-python/minigraf_graph_store.py` | `MinigrafGraphStore` class + `main()` |
| Modify | `tests/langchain_integrations.py` | Add two static smoke tests |
| Modify | `README.md` | Add LlamaIndex Integration section |

---

## Background: Key Minigraf Behaviours

Before implementing, understand these verified behaviours (all confirmed against Minigraf 1.1.1):

- **EAV facts**: `(transact [[:myapp :depends-on "pydantic==1.10"]])` — entity and attribute are keywords (`:name`), value is a JSON string.
- **Multi-valued attributes**: Multiple values for the same `(entity, attribute)` coexist when asserted in separate `execute` calls. Asserting the same `(entity, attribute, value)` twice is idempotent.
- **Retract**: `(retract [[:myapp :depends-on "pydantic==1.10"]])` removes a specific triple. History is preserved.
- **Temporal queries**: `:as-of N` goes **inside** the query, where `N` is a 1-based transaction ordinal that increments with every `transact` or `retract` call: `(query [:find ?o :as-of 3 :where [:myapp :depends-on ?o]])`.
- **Attribute format in results**: Attribute values returned from queries include the `:` prefix (e.g. `:depends-on`). Strip it for LlamaIndex consumers.

---

## Task 1: Create `requirements.txt` (TDD)

**Files:**
- Create: `integrations/llamaindex-python/requirements.txt`
- Modify: `tests/langchain_integrations.py`

- [ ] **Step 1: Write the failing test**

Append to `tests/langchain_integrations.py`:

```python
def test_llamaindex_integration_pins_dependencies():
    requirements = (
        ROOT / "integrations" / "llamaindex-python" / "requirements.txt"
    ).read_text()

    assert "minigraf==1.1.1" in requirements
    assert "llama-index-core" in requirements
```

- [ ] **Step 2: Run the test — verify it fails**

```bash
pytest tests/langchain_integrations.py::test_llamaindex_integration_pins_dependencies -v
```

Expected: `FAILED` with `FileNotFoundError` (file does not exist yet).

- [ ] **Step 3: Create `requirements.txt`**

Create `integrations/llamaindex-python/requirements.txt`:

```
minigraf==1.1.1
llama-index-core
```

- [ ] **Step 4: Run the test — verify it passes**

```bash
pytest tests/langchain_integrations.py::test_llamaindex_integration_pins_dependencies -v
```

Expected: `PASSED`.

- [ ] **Step 5: Commit**

```bash
git add integrations/llamaindex-python/requirements.txt tests/langchain_integrations.py
git commit -m "test: add LlamaIndex requirements.txt smoke test and pin dependencies"
```

---

## Task 2: Implement `MinigrafGraphStore` and `main()` (TDD)

**Files:**
- Create: `integrations/llamaindex-python/minigraf_graph_store.py`
- Modify: `tests/langchain_integrations.py`

- [ ] **Step 1: Write the failing test**

Append to `tests/langchain_integrations.py`:

```python
def test_llamaindex_example_documents_expected_output():
    example = (
        ROOT / "integrations" / "llamaindex-python" / "minigraf_graph_store.py"
    ).read_text()

    assert "class MinigrafGraphStore(SimpleGraphStore)" in example
    assert "myapp depends-on: pydantic==2.0, requests==2.28" in example
    assert "requests depends-on: urllib3==1.26" in example
    assert "myapp at tx 3 depended-on: pydantic==1.10, requests==2.28" in example
```

- [ ] **Step 2: Run the test — verify it fails**

```bash
pytest tests/langchain_integrations.py::test_llamaindex_example_documents_expected_output -v
```

Expected: `FAILED` with `FileNotFoundError`.

- [ ] **Step 3: Write `minigraf_graph_store.py`**

Create `integrations/llamaindex-python/minigraf_graph_store.py`:

```python
from __future__ import annotations

import json
import re
from typing import Any, Dict, List, Optional

from llama_index.core.graph_stores import SimpleGraphStore
from minigraf import MiniGrafDb


# SNAPSHOT_TX is the transaction ordinal after Beat 1 ingestion (3 upserts = tx 1, 2, 3).
# Used by main() to demonstrate :as-of historical queries.
SNAPSHOT_TX = 3

# EXPECTED_OUTPUT documents the stdout produced by main(). Also checked by static tests.
EXPECTED_OUTPUT = """\
myapp depends-on: pydantic==2.0, requests==2.28
requests depends-on: urllib3==1.26
myapp at tx 3 depended-on: pydantic==1.10, requests==2.28"""


def _to_kw(s: str) -> str:
    """Convert a plain string to a Minigraf keyword: 'depends-on' -> ':depends-on'."""
    slug = re.sub(r"[^a-z0-9-]", "-", s.lower()).strip("-")
    return f":{slug}"


def _from_kw(s: str) -> str:
    """Strip the leading colon returned by Minigraf attribute queries: ':depends-on' -> 'depends-on'."""
    return s.lstrip(":")


class MinigrafGraphStore(SimpleGraphStore):
    """LlamaIndex SimpleGraphStore backed by an in-memory Minigraf database.

    Triplets (subject, predicate, object) map directly onto Minigraf's native
    entity-attribute-value model:
      subject   -> Minigraf keyword entity  e.g. :myapp
      predicate -> Minigraf attribute keyword e.g. :depends-on
      object    -> JSON-quoted string value   e.g. "pydantic==1.10"

    Multiple values for the same (entity, attribute) pair coexist across
    separate transactions, which is the correct semantics for a graph store
    where one node can have many outgoing edges of the same type.
    """

    def __init__(self) -> None:
        self._db = MiniGrafDb.open_in_memory()

    @property
    def client(self) -> MiniGrafDb:
        return self._db

    @property
    def schema(self) -> str:
        return ""

    def get(self, subj: str) -> List[List[str]]:
        """Return all [relation, object] pairs for the given subject."""
        kw_subj = _to_kw(subj)
        result = json.loads(
            self._db.execute(f"(query [:find ?r ?o :where [{kw_subj} ?r ?o]])")
        )
        return [[_from_kw(r), o] for r, o in result["results"]]

    def get_rel_map(
        self,
        subjs: Optional[List[str]] = None,
        depth: int = 2,
        limit: int = 30,
    ) -> Dict[str, List[List[str]]]:
        """Return a map of subject -> [[relation, object], ...] for each subject."""
        if subjs is None:
            return {}
        return {subj: self.get(subj) for subj in subjs}

    def upsert_triplet(self, subj: str, rel: str, obj: str) -> None:
        """Assert a (subject, relation, object) triplet. Idempotent for existing triplets."""
        self._db.execute(
            f"(transact [[{_to_kw(subj)} {_to_kw(rel)} {json.dumps(obj)}]])"
        )

    def delete(self, subj: str, rel: str, obj: str) -> None:
        """Retract a specific (subject, relation, object) triplet. History is preserved."""
        self._db.execute(
            f"(retract [[{_to_kw(subj)} {_to_kw(rel)} {json.dumps(obj)}]])"
        )

    def query(self, query: str, param_map: Optional[Dict[str, Any]] = None) -> Any:
        """Execute a raw Datalog query string against the Minigraf database."""
        return self._db.execute(query)

    def get_schema(self, refresh: bool = False) -> str:
        return ""

    def persist(self, persist_path: str = "./storage/graph_store.json", fs=None) -> None:
        pass


def main() -> None:
    store = MinigrafGraphStore()

    # Beat 1: ingest v1 tech stack — transactions 1, 2, 3
    store.upsert_triplet("myapp", "depends-on", "pydantic==1.10")
    store.upsert_triplet("myapp", "depends-on", "requests==2.28")
    store.upsert_triplet("requests", "depends-on", "urllib3==1.26")
    # SNAPSHOT_TX == 3 at this point

    # Beat 2: upgrade pydantic — transactions 4 (retract), 5 (assert)
    store.delete("myapp", "depends-on", "pydantic==1.10")
    store.upsert_triplet("myapp", "depends-on", "pydantic==2.0")

    # Beat 3: query current state via get_rel_map
    rel_map = store.get_rel_map(["myapp", "requests"])
    for node in ["myapp", "requests"]:
        by_rel: Dict[str, List[str]] = {}
        for rel, obj in rel_map[node]:
            by_rel.setdefault(rel, []).append(obj)
        for rel, objs in sorted(by_rel.items()):
            print(f"{node} {rel}: {', '.join(sorted(objs))}")

    # Beat 4: query historical state using :as-of transaction ordinal
    hist = json.loads(
        store.query(
            f"(query [:find ?o :as-of {SNAPSHOT_TX} :where [:myapp :depends-on ?o]])"
        )
    )
    hist_vals = ", ".join(sorted(r[0] for r in hist["results"]))
    print(f"myapp at tx {SNAPSHOT_TX} depended-on: {hist_vals}")


if __name__ == "__main__":
    main()
```

- [ ] **Step 4: Run the static test — verify it passes**

```bash
pytest tests/langchain_integrations.py::test_llamaindex_example_documents_expected_output -v
```

Expected: `PASSED`.

- [ ] **Step 5: Run the full test suite — verify no regressions**

```bash
pytest tests/langchain_integrations.py -v
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add integrations/llamaindex-python/minigraf_graph_store.py tests/langchain_integrations.py
git commit -m "feat: add MinigrafGraphStore LlamaIndex integration"
```

---

## Task 3: Verify end-to-end execution

**Files:** No changes — this task runs the script and confirms the live output.

- [ ] **Step 1: Set up a virtual environment**

```bash
python3 -m venv .venv
. .venv/bin/activate
pip install -r integrations/llamaindex-python/requirements.txt
```

- [ ] **Step 2: Run the script**

```bash
python integrations/llamaindex-python/minigraf_graph_store.py
```

Expected output (exact):

```
myapp depends-on: pydantic==2.0, requests==2.28
requests depends-on: urllib3==1.26
myapp at tx 3 depended-on: pydantic==1.10, requests==2.28
```

If output does not match, check:
- Are all five `execute` calls in `main()` each in their own `transact`/`retract`? (`SNAPSHOT_TX = 3` requires exactly 3 separate transact calls in Beat 1.)
- Is the `:as-of` value correct? Count each `execute("(transact ...)")` and `execute("(retract ...)")` call in `main()` before Beat 4 — there must be exactly 3 before the snapshot.

---

## Task 4: Update README

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Add the LlamaIndex Integration section**

In `README.md`, append the following section after the `## GraphRAG Integration` section:

```markdown
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

\`\`\`sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r integrations/llamaindex-python/requirements.txt
\`\`\`

Run:

\`\`\`sh
python integrations/llamaindex-python/minigraf_graph_store.py
\`\`\`

Expected output:

\`\`\`text
myapp depends-on: pydantic==2.0, requests==2.28
requests depends-on: urllib3==1.26
myapp at tx 3 depended-on: pydantic==1.10, requests==2.28
\`\`\`

**Temporal model:** Each `upsert_triplet` call is one Minigraf transaction. After three
ingestion calls, `SNAPSHOT_TX = 3`. Retracting and re-asserting a dependency advances the
transaction counter. `(query [:find ?o :as-of 3 :where [:myapp :depends-on ?o]])` returns
the dependency set as it existed after transaction 3 — before the upgrade.
```

- [ ] **Step 2: Run the full test suite one final time**

```bash
pytest tests/langchain_integrations.py -v
```

Expected: all tests pass.

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: add LlamaIndex integration section to README"
```

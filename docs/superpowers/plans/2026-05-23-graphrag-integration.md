# GraphRAG + Minigraf Integration Example — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a runnable Python GraphRAG example that uses ChromaDB for semantic retrieval and Minigraf entity UUIDs as the bridge to structured graph traversal.

**Architecture:** A single script populates Minigraf with a six-node concept graph and ChromaDB with the same entities (keyed by Minigraf UUID), then demonstrates end-to-end: semantic query → UUID retrieval → graph traversal. Multi-valued `:related-to` edges are stored as edge entities (same pattern as `minigraf-algorithms` tests). Static-only CI test follows the existing LangChain pattern.

**Tech Stack:** Python, `minigraf==1.1.1`, `chromadb` (in-memory, default embedding function)

---

## File Map

| Action | Path | Responsibility |
|--------|------|----------------|
| Create | `integrations/graphrag-python/requirements.txt` | Pinned dependencies |
| Create | `integrations/graphrag-python/graphrag_minigraf.py` | Runnable GraphRAG example |
| Modify | `tests/langchain_integrations.py` | Static content assertions for new files |
| Modify | `README.md` | GraphRAG section under LangChain Integrations |

---

### Task 1: Add static tests for the new integration files

**Files:**
- Modify: `tests/langchain_integrations.py`

- [ ] **Step 1: Append the two static tests**

Open `tests/langchain_integrations.py` and add at the end:

```python
def test_graphrag_integration_pins_dependencies():
    requirements = (
        ROOT / "integrations" / "graphrag-python" / "requirements.txt"
    ).read_text()

    assert "minigraf==1.1.1" in requirements
    assert "chromadb" in requirements


def test_graphrag_example_documents_expected_output():
    example = (
        ROOT / "integrations" / "graphrag-python" / "graphrag_minigraf.py"
    ).read_text()

    assert "def main()" in example
    assert 'Query: "storing time-varying relationships"' in example
    assert "Match: temporal-data" in example
    assert "Related: graph-db, knowledge-graph" in example
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd /home/aditya/workspaces/rustrover/minigraf-examples
python -m pytest tests/langchain_integrations.py::test_graphrag_integration_pins_dependencies \
                 tests/langchain_integrations.py::test_graphrag_example_documents_expected_output -v
```

Expected: both FAIL with `FileNotFoundError` or `AssertionError` (files don't exist yet).

- [ ] **Step 3: Commit the failing tests**

```bash
git add tests/langchain_integrations.py
git commit -m "test: add static assertions for graphrag-python integration"
```

---

### Task 2: Create requirements.txt

**Files:**
- Create: `integrations/graphrag-python/requirements.txt`

- [ ] **Step 1: Create the file**

```
minigraf==1.1.1
chromadb
```

- [ ] **Step 2: Run the first static test to verify it passes**

```bash
python -m pytest tests/langchain_integrations.py::test_graphrag_integration_pins_dependencies -v
```

Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add integrations/graphrag-python/requirements.txt
git commit -m "feat: add graphrag-python requirements.txt"
```

---

### Task 3: Implement graphrag_minigraf.py

**Files:**
- Create: `integrations/graphrag-python/graphrag_minigraf.py`

- [ ] **Step 1: Write the script**

```python
from __future__ import annotations

import json

import chromadb
from minigraf import MiniGrafDb


EXPECTED_OUTPUT = """\
Query: "storing time-varying relationships"
Match: temporal-data
Related: graph-db, knowledge-graph"""

# Six concept entities: (minigraf keyword, name, description)
_CONCEPTS = [
    (":graph-db", "graph-db",
     "A database that stores and queries data as nodes and edges."),
    (":vector-search", "vector-search",
     "Retrieval of items by semantic similarity using embedding vectors."),
    (":temporal-data", "temporal-data",
     "Data that records how facts change over time with transaction history."),
    (":knowledge-graph", "knowledge-graph",
     "A graph of entities and relationships used for structured reasoning."),
    (":embeddings", "embeddings",
     "Dense numerical representations of text used in semantic search."),
    (":rag", "rag",
     "Retrieval-augmented generation: combine retrieval with language models."),
]

# Directed :related-to edges stored as edge entities (supports multi-valued).
# Each tuple is (from-keyword, to-keyword).
_EDGES = [
    (":graph-db", ":knowledge-graph"),
    (":graph-db", ":temporal-data"),
    (":vector-search", ":embeddings"),
    (":vector-search", ":rag"),
    (":rag", ":knowledge-graph"),
    (":temporal-data", ":graph-db"),
    (":temporal-data", ":knowledge-graph"),
    (":knowledge-graph", ":graph-db"),
]


def _query_results(db: MiniGrafDb, query: str) -> list[list]:
    return json.loads(db.execute(query))["results"]


def _populate_minigraf(db: MiniGrafDb) -> None:
    facts = []
    for keyword, name, description in _CONCEPTS:
        facts.append(f'[{keyword} :concept/name "{name}"]')
        facts.append(f'[{keyword} :concept/description "{description}"]')
    for i, (from_kw, to_kw) in enumerate(_EDGES):
        edge = f":rel-{i}"
        facts.append(f"[{edge} :related-to/from {from_kw}]")
        facts.append(f"[{edge} :related-to/to {to_kw}]")
    db.execute(f"(transact [{' '.join(facts)}])")


def _populate_chromadb(db: MiniGrafDb, collection: chromadb.Collection) -> None:
    # The UUID Minigraf assigns to each keyword entity is used as the ChromaDB
    # document ID — this is the bridge between vector retrieval and graph traversal.
    ids = []
    documents = []
    for _, name, description in _CONCEPTS:
        rows = _query_results(db, f'(query [:find ?e :where [?e :concept/name "{name}"]])')
        uuid_str = str(rows[0][0])
        ids.append(uuid_str)
        documents.append(description)
    collection.add(ids=ids, documents=documents)


def _name_for_uuid(db: MiniGrafDb, uuid_str: str) -> str:
    rows = _query_results(
        db,
        f'(query [:find ?name :where [#uuid "{uuid_str}" :concept/name ?name]])',
    )
    return str(rows[0][0])


def _neighbour_names(db: MiniGrafDb, uuid_str: str) -> list[str]:
    rows = _query_results(
        db,
        f'(query [:find ?name'
        f' :where [?edge :related-to/from #uuid "{uuid_str}"]'
        f'        [?edge :related-to/to ?target]'
        f'        [?target :concept/name ?name]])',
    )
    return sorted(str(row[0]) for row in rows)


def main() -> None:
    db = MiniGrafDb.open_in_memory()

    # Step 1: populate Minigraf with concept graph
    _populate_minigraf(db)

    # Step 2: populate ChromaDB — UUIDs from Minigraf are the document IDs
    client = chromadb.Client()
    collection = client.create_collection("concepts")
    _populate_chromadb(db, collection)

    # Step 3: semantic retrieval
    query_text = "storing time-varying relationships"
    results = collection.query(query_texts=[query_text], n_results=1)
    matched_uuid = results["ids"][0][0]

    # Step 4: graph traversal anchored on the retrieved UUID
    matched_name = _name_for_uuid(db, matched_uuid)
    neighbours = _neighbour_names(db, matched_uuid)

    print(f'Query: "{query_text}"')
    print(f"Match: {matched_name}")
    print(f"Related: {', '.join(neighbours)}")


if __name__ == "__main__":
    main()
```

- [ ] **Step 2: Run the static test to verify it passes**

```bash
python -m pytest tests/langchain_integrations.py::test_graphrag_example_documents_expected_output -v
```

Expected: PASS.

- [ ] **Step 3: Run all static tests together**

```bash
python -m pytest tests/langchain_integrations.py -v
```

Expected: all 6 tests PASS.

- [ ] **Step 4: Smoke-run the script (requires chromadb installed)**

```bash
cd /home/aditya/workspaces/rustrover/minigraf-examples
python -m venv .venv-graphrag
. .venv-graphrag/bin/activate
pip install -r integrations/graphrag-python/requirements.txt chromadb
python integrations/graphrag-python/graphrag_minigraf.py
deactivate
```

Expected output (first run downloads ~80 MB model):
```
Query: "storing time-varying relationships"
Match: temporal-data
Related: graph-db, knowledge-graph
```

If the output differs, the description text for `:temporal-data` may need tuning to score above the other concepts for this query. Adjust the description and re-run until the output matches.

- [ ] **Step 5: Commit**

```bash
git add integrations/graphrag-python/graphrag_minigraf.py
git commit -m "feat: add graphrag-python integration example"
```

---

### Task 4: Update README.md

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Add the GraphRAG subsection**

In `README.md`, find the end of the `### LangChain.js` section (after its expected output
block) and append the following. The inner code fences use `sh` / `text` tags — write them
as literal triple-backtick blocks in the file:

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

(triple-backtick)sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r integrations/graphrag-python/requirements.txt
(triple-backtick)

Note: ChromaDB downloads the `all-MiniLM-L6-v2` embedding model (~80 MB) on first run.

Run:

(triple-backtick)sh
python integrations/graphrag-python/graphrag_minigraf.py
(triple-backtick)

Expected output:

(triple-backtick)text
Query: "storing time-varying relationships"
Match: temporal-data
Related: graph-db, knowledge-graph
(triple-backtick)
```

Replace each `(triple-backtick)` with three literal backtick characters when editing the file.

- [ ] **Step 2: Verify README renders correctly (spot check)**

```bash
grep -A 5 "GraphRAG" README.md
```

Expected: shows the new section heading and first few lines.

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: add GraphRAG integration section to README"
```

---

### Task 5: Final check

- [ ] **Step 1: Run the full Rust test suite**

```bash
cargo test --verbose
```

Expected: all tests pass.

- [ ] **Step 2: Run all static Python tests**

```bash
python -m pytest tests/langchain_integrations.py -v
```

Expected: all 6 tests pass.

- [ ] **Step 3: Verify git log looks clean**

```bash
git log --oneline -5
```

Expected: 4 new commits since `main` — tests, requirements, implementation, README.

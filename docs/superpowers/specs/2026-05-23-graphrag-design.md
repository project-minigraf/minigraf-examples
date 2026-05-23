# GraphRAG + Minigraf Integration Example — Design

**Issue:** [#4 post-1.0: add GraphRAG integration example](https://github.com/project-minigraf/minigraf-examples/issues/4)
**Date:** 2026-05-23

## Goal

Add a runnable Python example that demonstrates GraphRAG: use a vector store to retrieve
entities by semantic similarity, then use their Minigraf UUIDs to anchor structured graph
traversal. The UUID is the explicit bridge between retrieval and graph reasoning.

## Location and structure

```
integrations/graphrag-python/
  graphrag_minigraf.py   # single runnable script
  requirements.txt       # pinned dependencies
```

Follows the pattern of `integrations/langchain-python/` exactly. A static-check test is
added to `tests/langchain_integrations.py` (content assertions only, no live execution in
CI — matching the existing LangChain tests).

The root `README.md` gains a "GraphRAG" subsection under "LangChain Integrations", following
the same format: install, run, expected output.

## Scenario

A small knowledge graph of six technical concepts stored in Minigraf:

| Entity keyword  | Name              | Description (used as embedding text)                                      |
|-----------------|-------------------|---------------------------------------------------------------------------|
| `:graph-db`     | graph-db          | A database that stores and queries data as nodes and edges.               |
| `:vector-search`| vector-search     | Retrieval of items by semantic similarity using embedding vectors.        |
| `:temporal-data`| temporal-data     | Data that records how facts change over time with transaction history.    |
| `:knowledge-graph` | knowledge-graph| A graph of entities and relationships used for structured reasoning.      |
| `:embeddings`   | embeddings        | Dense numerical representations of text used in semantic search.         |
| `:rag`          | rag               | Retrieval-augmented generation: combine retrieval with language models.   |

Relationships (`:related-to` edges):
- `graph-db` → `knowledge-graph`, `temporal-data`
- `vector-search` → `embeddings`, `rag`
- `rag` → `knowledge-graph`
- `temporal-data` → `graph-db`, `knowledge-graph`
- `knowledge-graph` → `graph-db`

## Data flow

```
1. Populate Minigraf
   Transact all six concept entities with :concept/name, :concept/description,
   and :related-to edges.

2. Populate ChromaDB
   For each entity:
     - Query Minigraf for its UUID (via [?e :concept/name ?name])
     - Add a document to a ChromaDB in-memory collection:
         id       = str(uuid)          ← the Minigraf entity UUID
         document = description text
   ChromaDB's default embedding function (sentence-transformers/all-MiniLM-L6-v2)
   vectorises each description automatically.

3. Retrieve by similarity
   Run a natural-language query against ChromaDB (top-1 result).
   The returned document ID is the Minigraf UUID of the matched entity.
   A comment in the code calls this out explicitly as the bridge.

4. Traverse the graph
   Use the UUID to query Minigraf:
     - Fetch :concept/name for the matched entity
     - Fetch all :related-to neighbours and their names
   Print the match and its neighbours.
```

## Expected output

```
Query: "storing time-varying relationships"
Match: temporal-data
Related: graph-db, knowledge-graph
```

This output is hardcoded as `EXPECTED_OUTPUT` in the script and asserted in the static test.

## Dependencies

`requirements.txt`:
```
minigraf==1.1.1
chromadb
```

No API key required. ChromaDB downloads `all-MiniLM-L6-v2` (~80 MB) on first run.
This is documented once in the README install instructions.

## CI strategy

A new test is added to `tests/langchain_integrations.py`, run via pytest separately from
the Rust CI workflow. The new test asserts:
- `requirements.txt` contains `minigraf==1.1.1` and `chromadb`
- `graphrag_minigraf.py` contains the expected class/function structure and expected output string

No live ChromaDB or embedding model execution in CI. Matches the established pattern for
integration examples in this repo.

## Not in scope

- Pinning `chromadb` to a specific version (leave unpinned; add a pin if CI shows instability)
- Multiple retrieval results or re-ranking
- Persistent ChromaDB storage (in-memory only)
- LangChain wiring (this example is standalone, not a LangChain component)
- Weighted or scored graph traversal (use `minigraf-algorithms` if needed in future)

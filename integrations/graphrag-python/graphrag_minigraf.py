from __future__ import annotations

import json

import chromadb
from minigraf import MiniGrafDb


# EXPECTED_OUTPUT documents the stdout produced by this script. It is also
# checked by the static tests in tests/langchain_integrations.py.
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
    client = chromadb.EphemeralClient()
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

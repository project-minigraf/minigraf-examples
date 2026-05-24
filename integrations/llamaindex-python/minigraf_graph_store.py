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
    """Convert a plain string to a Minigraf keyword: 'depends-on' -> ':depends-on'.

    Input is lowercased and any character outside [a-z0-9-] (including '/', '.', '_')
    is replaced with '-'. Callers must ensure distinct inputs produce distinct keywords.
    """
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
        """Return a map of subject -> [[relation, object], ...] for each subject.

        depth and limit are accepted for interface compatibility but are not applied;
        all edges for each subject are returned.
        """
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

    def persist(self, persist_path: str = "./storage/graph_store.json", fs: Optional[Any] = None) -> None:
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

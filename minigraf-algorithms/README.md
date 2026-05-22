# minigraf-algorithms

Graph algorithms for Minigraf ecosystem crates.

This crate is intentionally separate from `minigraf` core. Algorithms are
application-level utilities: useful for graph-shaped data, but not required for
the embedded database engine itself. Keeping them here lets the core crate stay
small while users can opt into traversal helpers when they need them.

## Current API

- `GraphView`: read-only graph access trait used by algorithms.
- `EdgeList`: in-memory directed graph for tests and preloaded data.
- `MinigrafGraph`: adapter that loads graph edges from Minigraf query results.
- `reachable`: breadth-first reachability from a start node.

## Minigraf Edge Shape

For multi-edge graphs, prefer edge entities:

```text
[:edge-1 :edge/from :a]
[:edge-1 :edge/to :b]
[:edge-2 :edge/from :a]
[:edge-2 :edge/to :c]
```

Then load them with:

```rust
let graph = minigraf_algorithms::MinigrafGraph::load_edge_entities(
    &db,
    ":edge/from",
    ":edge/to",
)?;
```

`MinigrafGraph::load(&db, ":edge")` is also available for simple functional
relationships shaped as `[?from :edge ?to]`.

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

### Algorithms

Each algorithm is a free function over `&impl GraphView`, so it works the same
on an `EdgeList` or a `MinigrafGraph`. `V` is the number of nodes and `E` is
the number of edges.

| Function | What it returns | Time |
| --- | --- | --- |
| `reachable(graph, start)` | Nodes reachable from `start`, breadth-first. | O(V + E) |
| `depth_first(graph, start)` | Nodes reachable from `start`, depth-first (preorder). | O(V + E) |
| `shortest_path(graph, from, to)` | Minimum-hop path, or `None` if there is no path. | O(V + E) |
| `topological_sort(graph)` | Nodes ordered so every edge points forward. Returns an error if the graph has a cycle. | O(V + E) |
| `connected_components(graph)` | Weakly connected components. Edge direction is ignored. | O(V + E) |

Edges are unweighted. `shortest_path` finds the path with the fewest edges.

### Implementing `GraphView`

`GraphView` has two methods:

- `nodes()` returns every node, including nodes with no outgoing edges.
  Whole-graph algorithms (`topological_sort`, `connected_components`) use it,
  and use its order to break ties.
- `outgoing(node)` returns the direct successors of `node`.

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

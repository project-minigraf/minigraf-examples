#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Graph algorithms for data stored in Minigraf.
//!
//! This crate intentionally lives outside `minigraf` core. Algorithms are
//! ecosystem utilities: they are useful for graph-shaped applications, but they
//! should not enlarge the embedded database API or couple storage internals to
//! application-level traversal choices.

use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::{Result, bail};
use minigraf::{Minigraf, QueryResult, Value};
use uuid::Uuid;

/// Read-only graph access used by algorithms in this crate.
pub trait GraphView {
    /// Returns every node in the graph, including nodes with no outgoing edges.
    ///
    /// Algorithms that visit the whole graph use this order to break ties, so
    /// a stable order gives stable results.
    fn nodes(&self) -> Vec<Value>;

    /// Returns the outgoing neighbors for `node`.
    fn outgoing(&self, node: &Value) -> Vec<Value>;
}

/// In-memory directed edge list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EdgeList {
    nodes: Vec<Value>,
    adjacency: HashMap<Value, Vec<Value>>,
}

impl EdgeList {
    /// Builds an edge list from `(from, to)` pairs.
    pub fn from_edges<I>(edges: I) -> Self
    where
        I: IntoIterator<Item = (Value, Value)>,
    {
        let mut seen = HashSet::new();
        let mut nodes = Vec::new();
        let mut adjacency: HashMap<Value, Vec<Value>> = HashMap::new();
        for (from, to) in edges {
            for node in [&from, &to] {
                if seen.insert(node.clone()) {
                    nodes.push(node.clone());
                }
            }
            adjacency.entry(from).or_default().push(to);
        }
        Self { nodes, adjacency }
    }
}

impl GraphView for EdgeList {
    fn nodes(&self) -> Vec<Value> {
        self.nodes.clone()
    }

    fn outgoing(&self, node: &Value) -> Vec<Value> {
        self.adjacency.get(node).cloned().unwrap_or_default()
    }
}

/// A graph loaded from Minigraf facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinigrafGraph {
    edges: EdgeList,
}

impl MinigrafGraph {
    /// Loads all facts matching `[?from edge_attribute ?to]` into an in-memory graph.
    ///
    /// # Node normalization
    ///
    /// `Value::Keyword` values returned by the query are converted to `Value::Ref`
    /// using a derived UUID. Use the values returned by the graph itself or obtained
    /// by querying the database when constructing start nodes for traversal — do not
    /// pass manually constructed `Value::Keyword` literals to `reachable` or similar
    /// functions, as they will not match the normalized keys.
    pub fn load(db: &Minigraf, edge_attribute: &str) -> Result<Self> {
        validate_attribute(edge_attribute)?;

        let query = format!("(query [:find ?from ?to :where [?from {edge_attribute} ?to]])");
        let result = db.execute(&query)?;
        let QueryResult::QueryResults { results, .. } = result else {
            bail!("expected query results when loading graph edges");
        };

        Ok(Self {
            edges: parse_edge_results(results)?,
        })
    }

    /// Loads edge entities with separate source and target attributes.
    ///
    /// This supports multi-edge graph data such as:
    ///
    /// ```text
    /// [:edge-1 :edge/from :a]
    /// [:edge-1 :edge/to :b]
    /// [:edge-2 :edge/from :a]
    /// [:edge-2 :edge/to :c]
    /// ```
    ///
    /// # Node normalization
    ///
    /// `Value::Keyword` values returned by the query are converted to `Value::Ref`
    /// using a derived UUID. Use the values returned by the graph itself or obtained
    /// by querying the database when constructing start nodes for traversal — do not
    /// pass manually constructed `Value::Keyword` literals to `reachable` or similar
    /// functions, as they will not match the normalized keys.
    pub fn load_edge_entities(
        db: &Minigraf,
        from_attribute: &str,
        to_attribute: &str,
    ) -> Result<Self> {
        validate_attribute(from_attribute)?;
        validate_attribute(to_attribute)?;

        let query = format!(
            "(query [:find ?from ?to :where [?edge {from_attribute} ?from] [?edge {to_attribute} ?to]])"
        );
        let result = db.execute(&query)?;
        let QueryResult::QueryResults { results, .. } = result else {
            bail!("expected query results when loading graph edges");
        };

        Ok(Self {
            edges: parse_edge_results(results)?,
        })
    }
}

impl GraphView for MinigrafGraph {
    fn nodes(&self) -> Vec<Value> {
        self.edges.nodes()
    }

    fn outgoing(&self, node: &Value) -> Vec<Value> {
        self.edges.outgoing(node)
    }
}

/// Returns every node reachable from `start` in breadth-first order.
///
/// The result does not include `start` unless a cycle leads back to it.
///
/// Time complexity: O(V + E) for the reachable part of the graph.
pub fn reachable<G>(graph: &G, start: &Value) -> Vec<Value>
where
    G: GraphView,
{
    let mut seen = HashSet::from([start.clone()]);
    let mut queue = VecDeque::from([start.clone()]);
    let mut result = Vec::new();

    while let Some(node) = queue.pop_front() {
        for neighbor in graph.outgoing(&node) {
            if seen.insert(neighbor.clone()) {
                queue.push_back(neighbor.clone());
                result.push(neighbor);
            }
        }
    }

    result
}

/// Returns every node reachable from `start` in depth-first (preorder) order.
///
/// Neighbors are visited in the order `GraphView::outgoing` returns them. Like
/// [`reachable`], the result does not include `start` unless a cycle leads back
/// to it.
///
/// Time complexity: O(V + E) for the reachable part of the graph.
pub fn depth_first<G>(graph: &G, start: &Value) -> Vec<Value>
where
    G: GraphView,
{
    let mut seen = HashSet::from([start.clone()]);
    let mut stack: Vec<Value> = graph.outgoing(start).into_iter().rev().collect();
    let mut result = Vec::new();

    while let Some(node) = stack.pop() {
        if !seen.insert(node.clone()) {
            continue;
        }
        stack.extend(
            graph
                .outgoing(&node)
                .into_iter()
                .rev()
                .filter(|neighbor| !seen.contains(neighbor)),
        );
        result.push(node);
    }

    result
}

/// Returns a minimum-hop path from `from` to `to`, following edge direction.
///
/// The path starts with `from` and ends with `to`. If `from == to`, the path is
/// `[from]`. Returns `None` if `to` is not reachable from `from`. Edges are
/// unweighted, so the path has the fewest edges, not the lowest cost.
///
/// Time complexity: O(V + E) for the reachable part of the graph.
pub fn shortest_path<G>(graph: &G, from: &Value, to: &Value) -> Option<Vec<Value>>
where
    G: GraphView,
{
    if from == to {
        return Some(vec![from.clone()]);
    }

    let mut parent: HashMap<Value, Value> = HashMap::new();
    let mut queue = VecDeque::from([from.clone()]);

    while let Some(node) = queue.pop_front() {
        for neighbor in graph.outgoing(&node) {
            if neighbor == *from || parent.contains_key(&neighbor) {
                continue;
            }
            parent.insert(neighbor.clone(), node.clone());
            if neighbor == *to {
                let mut path = vec![neighbor];
                while let Some(previous) = parent.get(path.last()?) {
                    path.push(previous.clone());
                }
                path.reverse();
                return Some(path);
            }
            queue.push_back(neighbor);
        }
    }

    None
}

/// Orders every node so that each edge points from an earlier node to a later one.
///
/// Uses Kahn's algorithm. When several nodes are ready at the same time, they
/// come out in `GraphView::nodes` order. Returns an error if the graph has a
/// cycle, because no such order exists.
///
/// Time complexity: O(V + E).
pub fn topological_sort<G>(graph: &G) -> Result<Vec<Value>>
where
    G: GraphView,
{
    let nodes = graph.nodes();
    let mut in_degree: HashMap<Value, usize> = nodes.iter().map(|node| (node.clone(), 0)).collect();
    for node in &nodes {
        for neighbor in graph.outgoing(node) {
            *in_degree.entry(neighbor).or_default() += 1;
        }
    }

    let mut queue: VecDeque<Value> = nodes
        .iter()
        .filter(|node| in_degree[*node] == 0)
        .cloned()
        .collect();
    let mut result = Vec::with_capacity(nodes.len());

    while let Some(node) = queue.pop_front() {
        for neighbor in graph.outgoing(&node) {
            let degree = in_degree
                .get_mut(&neighbor)
                .expect("every neighbor has an in-degree entry");
            *degree -= 1;
            if *degree == 0 {
                queue.push_back(neighbor);
            }
        }
        result.push(node);
    }

    if result.len() < in_degree.len() {
        let stuck = nodes
            .iter()
            .find(|node| in_degree[*node] > 0)
            .map(|node| format!("{node:?}"))
            .unwrap_or_else(|| "an unknown node".to_string());
        bail!("graph contains a cycle; topological sort is not possible (cycle involves {stuck})");
    }

    Ok(result)
}

/// Splits the graph into weakly connected components.
///
/// Edge direction is ignored: two nodes are in the same component if any path
/// joins them when every edge is treated as undirected. Components come out in
/// the order of their first node in `GraphView::nodes`, and nodes inside a
/// component come out in breadth-first order from that first node.
///
/// Time complexity: O(V + E).
pub fn connected_components<G>(graph: &G) -> Vec<Vec<Value>>
where
    G: GraphView,
{
    let nodes = graph.nodes();
    let mut undirected: HashMap<Value, Vec<Value>> = HashMap::new();
    for node in &nodes {
        for neighbor in graph.outgoing(node) {
            undirected
                .entry(node.clone())
                .or_default()
                .push(neighbor.clone());
            undirected.entry(neighbor).or_default().push(node.clone());
        }
    }

    let mut seen = HashSet::new();
    let mut components = Vec::new();

    for node in nodes {
        if !seen.insert(node.clone()) {
            continue;
        }
        let mut component = Vec::new();
        let mut queue = VecDeque::from([node]);
        while let Some(current) = queue.pop_front() {
            for neighbor in undirected.get(&current).into_iter().flatten() {
                if seen.insert(neighbor.clone()) {
                    queue.push_back(neighbor.clone());
                }
            }
            component.push(current);
        }
        components.push(component);
    }

    components
}

fn parse_edge_results(results: Vec<Vec<Value>>) -> Result<EdgeList> {
    let mut edges = Vec::with_capacity(results.len());
    for row in results {
        let [from, to]: [Value; 2] = row
            .try_into()
            .map_err(|_| anyhow::anyhow!("expected two columns for graph edge query"))?;
        edges.push((normalize_node(from), normalize_node(to)));
    }
    Ok(EdgeList::from_edges(edges))
}

fn validate_attribute(attribute: &str) -> Result<()> {
    if !attribute.starts_with(':') {
        bail!("edge attribute must be a Minigraf keyword like :edge or :graph/edge");
    }

    if attribute.len() == 1
        || !attribute
            .chars()
            .skip(1)
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '-' | '_' | '.'))
    {
        bail!("edge attribute contains unsupported characters");
    }

    Ok(())
}

fn normalize_node(value: Value) -> Value {
    match value {
        Value::Keyword(keyword) => Value::Ref(keyword_entity_id(&keyword)),
        other => other,
    }
}

fn keyword_entity_id(keyword: &str) -> Uuid {
    Uuid::parse_str(keyword.trim_start_matches(':'))
        .unwrap_or_else(|_| Uuid::new_v5(&Uuid::NAMESPACE_OID, keyword.as_bytes()))
}

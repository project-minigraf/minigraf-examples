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
    /// Returns the outgoing neighbors for `node`.
    fn outgoing(&self, node: &Value) -> Vec<Value>;
}

/// In-memory directed edge list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EdgeList {
    adjacency: HashMap<Value, Vec<Value>>,
}

impl EdgeList {
    /// Builds an edge list from `(from, to)` pairs.
    pub fn from_edges<I>(edges: I) -> Self
    where
        I: IntoIterator<Item = (Value, Value)>,
    {
        let mut adjacency: HashMap<Value, Vec<Value>> = HashMap::new();
        for (from, to) in edges {
            adjacency.entry(from).or_default().push(to);
        }
        Self { adjacency }
    }
}

impl GraphView for EdgeList {
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
    fn outgoing(&self, node: &Value) -> Vec<Value> {
        self.edges.outgoing(node)
    }
}

/// Returns every node reachable from `start` in breadth-first order.
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

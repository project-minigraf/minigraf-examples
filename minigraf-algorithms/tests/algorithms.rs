use minigraf::{Minigraf, QueryResult, Value};
use minigraf_algorithms::{EdgeList, GraphView, MinigrafGraph, reachable};

#[test]
fn edge_list_reports_outgoing_neighbors() {
    let graph = EdgeList::from_edges([
        (
            Value::Keyword(":a".to_string()),
            Value::Keyword(":b".to_string()),
        ),
        (
            Value::Keyword(":a".to_string()),
            Value::Keyword(":c".to_string()),
        ),
        (
            Value::Keyword(":b".to_string()),
            Value::Keyword(":d".to_string()),
        ),
    ]);

    assert_eq!(
        graph.outgoing(&Value::Keyword(":a".to_string())),
        vec![
            Value::Keyword(":b".to_string()),
            Value::Keyword(":c".to_string())
        ]
    );
}

#[test]
fn reachable_returns_breadth_first_nodes_without_revisiting_cycles() {
    let graph = EdgeList::from_edges([
        (
            Value::Keyword(":a".to_string()),
            Value::Keyword(":b".to_string()),
        ),
        (
            Value::Keyword(":a".to_string()),
            Value::Keyword(":c".to_string()),
        ),
        (
            Value::Keyword(":b".to_string()),
            Value::Keyword(":d".to_string()),
        ),
        (
            Value::Keyword(":c".to_string()),
            Value::Keyword(":d".to_string()),
        ),
        (
            Value::Keyword(":d".to_string()),
            Value::Keyword(":a".to_string()),
        ),
    ]);

    assert_eq!(
        reachable(&graph, &Value::Keyword(":a".to_string())),
        vec![
            Value::Keyword(":b".to_string()),
            Value::Keyword(":c".to_string()),
            Value::Keyword(":d".to_string())
        ]
    );
}

#[test]
fn minigraf_graph_loads_edges_for_attribute() {
    let db = Minigraf::in_memory().unwrap();
    db.execute(
        r#"(transact [[:edge-1 :edge/from :a] [:edge-1 :edge/to :b]
                      [:edge-2 :edge/from :a] [:edge-2 :edge/to :c]
                      [:edge-3 :edge/from :b] [:edge-3 :edge/to :d]
                      [:a :node/name "a"] [:b :node/name "b"]
                      [:c :node/name "c"] [:d :node/name "d"]])"#,
    )
    .unwrap();

    let graph = MinigrafGraph::load_edge_entities(&db, ":edge/from", ":edge/to").unwrap();
    let a = query_one(
        &db,
        r#"(query [:find ?node :where [?node :node/name "a"]])"#,
    );
    let b = query_one(
        &db,
        r#"(query [:find ?node :where [?node :node/name "b"]])"#,
    );
    let c = query_one(
        &db,
        r#"(query [:find ?node :where [?node :node/name "c"]])"#,
    );
    let d = query_one(
        &db,
        r#"(query [:find ?node :where [?node :node/name "d"]])"#,
    );

    let reachable = reachable(&graph, &a);
    assert_eq!(reachable.len(), 3);
    assert!(reachable.contains(&b));
    assert!(reachable.contains(&c));
    assert!(reachable.contains(&d));
    assert_eq!(reachable[2], d);
}

#[test]
fn minigraf_graph_load_simple_attribute() {
    let db = Minigraf::in_memory().unwrap();
    // Direct-attribute edge shape: [?from :parent ?to] — use a chain so each
    // node has at most one outgoing edge, avoiding single-valued attribute conflicts.
    db.execute(
        r#"(transact [[:a :node/name "a"] [:b :node/name "b"] [:c :node/name "c"]
                      [:a :parent :b] [:b :parent :c]])"#,
    )
    .unwrap();

    let graph = MinigrafGraph::load(&db, ":parent").unwrap();
    let a = query_one(&db, r#"(query [:find ?node :where [?node :node/name "a"]])"#);
    let b = query_one(&db, r#"(query [:find ?node :where [?node :node/name "b"]])"#);
    let c = query_one(&db, r#"(query [:find ?node :where [?node :node/name "c"]])"#);

    let reachable = reachable(&graph, &a);
    assert_eq!(reachable.len(), 2);
    assert!(reachable.contains(&b));
    assert!(reachable.contains(&c));
}

#[test]
fn load_rejects_attribute_without_leading_colon() {
    let db = Minigraf::in_memory().unwrap();
    assert!(MinigrafGraph::load(&db, "follows").is_err());
    assert!(MinigrafGraph::load_edge_entities(&db, "follows", ":to").is_err());
}

#[test]
fn load_rejects_bare_colon_attribute() {
    let db = Minigraf::in_memory().unwrap();
    assert!(MinigrafGraph::load(&db, ":").is_err());
}

#[test]
fn load_rejects_attribute_with_unsupported_characters() {
    let db = Minigraf::in_memory().unwrap();
    assert!(MinigrafGraph::load(&db, ":bad attr").is_err());
    assert!(MinigrafGraph::load(&db, ":bad@attr").is_err());
}

fn query_one(db: &Minigraf, query: &str) -> Value {
    let QueryResult::QueryResults { results, .. } = db.execute(query).unwrap() else {
        panic!("expected query results");
    };

    results[0][0].clone()
}

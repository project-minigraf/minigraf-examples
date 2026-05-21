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

fn query_one(db: &Minigraf, query: &str) -> Value {
    let QueryResult::QueryResults { results, .. } = db.execute(query).unwrap() else {
        panic!("expected query results");
    };

    results[0][0].clone()
}

use minigraf::{Minigraf, QueryResult, Value};
use minigraf_algorithms::{
    EdgeList, GraphView, MinigrafGraph, connected_components, depth_first, reachable,
    shortest_path, topological_sort,
};

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

fn kw(name: &str) -> Value {
    Value::Keyword(format!(":{name}"))
}

fn graph(edges: &[(&str, &str)]) -> EdgeList {
    EdgeList::from_edges(edges.iter().map(|(from, to)| (kw(from), kw(to))))
}

fn kws(names: &[&str]) -> Vec<Value> {
    names.iter().map(|name| kw(name)).collect()
}

/// Loads a graph from edge entities and returns it with a lookup for node values by name.
fn load_named_graph(edges: &[(&str, &str)]) -> (MinigrafGraph, impl Fn(&str) -> Value) {
    let db = Minigraf::in_memory().unwrap();
    let mut names: Vec<&str> = Vec::new();
    let mut facts = String::new();
    for (index, (from, to)) in edges.iter().enumerate() {
        facts.push_str(&format!(
            "[:edge-{index} :edge/from :{from}] [:edge-{index} :edge/to :{to}] "
        ));
        for name in [from, to] {
            if !names.contains(name) {
                names.push(name);
            }
        }
    }
    for name in &names {
        facts.push_str(&format!("[:{name} :node/name \"{name}\"] "));
    }
    db.execute(&format!("(transact [{facts}])")).unwrap();

    let graph = MinigrafGraph::load_edge_entities(&db, ":edge/from", ":edge/to").unwrap();
    let lookup = move |name: &str| {
        query_one(
            &db,
            &format!("(query [:find ?node :where [?node :node/name \"{name}\"]])"),
        )
    };
    (graph, lookup)
}

#[test]
fn edge_list_reports_all_nodes_in_first_seen_order() {
    let graph = graph(&[("a", "b"), ("c", "a"), ("b", "d")]);
    assert_eq!(graph.nodes(), kws(&["a", "b", "c", "d"]));
}

#[test]
fn depth_first_follows_each_branch_before_the_next() {
    let graph = graph(&[("a", "b"), ("a", "c"), ("b", "d"), ("c", "e"), ("d", "a")]);
    assert_eq!(depth_first(&graph, &kw("a")), kws(&["b", "d", "c", "e"]));
}

#[test]
fn depth_first_includes_start_only_when_a_cycle_returns_to_it() {
    let graph = graph(&[("a", "b")]);
    assert_eq!(depth_first(&graph, &kw("b")), Vec::<Value>::new());
    assert_eq!(depth_first(&graph, &kw("a")), kws(&["b"]));
}

#[test]
fn shortest_path_returns_fewest_hops() {
    let graph = graph(&[("a", "b"), ("b", "c"), ("c", "d"), ("a", "e"), ("e", "d")]);
    assert_eq!(
        shortest_path(&graph, &kw("a"), &kw("d")),
        Some(kws(&["a", "e", "d"]))
    );
}

#[test]
fn shortest_path_to_self_is_single_node() {
    let graph = graph(&[("a", "b")]);
    assert_eq!(shortest_path(&graph, &kw("a"), &kw("a")), Some(kws(&["a"])));
}

#[test]
fn shortest_path_follows_edge_direction() {
    let graph = graph(&[("a", "b"), ("b", "a"), ("c", "b")]);
    assert_eq!(shortest_path(&graph, &kw("a"), &kw("c")), None);
    assert_eq!(
        shortest_path(&graph, &kw("c"), &kw("a")),
        Some(kws(&["c", "b", "a"]))
    );
}

#[test]
fn topological_sort_orders_every_edge_forward() {
    let edges = [
        ("shirt", "tie"),
        ("tie", "jacket"),
        ("pants", "shoes"),
        ("pants", "belt"),
        ("belt", "jacket"),
        ("shirt", "belt"),
    ];
    let graph = graph(&edges);
    let order = topological_sort(&graph).unwrap();

    assert_eq!(order.len(), graph.nodes().len());
    let position = |name: &str| order.iter().position(|node| *node == kw(name)).unwrap();
    for (from, to) in edges {
        assert!(
            position(from) < position(to),
            "{from} must come before {to}"
        );
    }
}

#[test]
fn topological_sort_breaks_ties_in_node_order() {
    let graph = graph(&[("a", "c"), ("b", "c"), ("c", "d")]);
    assert_eq!(
        topological_sort(&graph).unwrap(),
        kws(&["a", "b", "c", "d"])
    );
}

#[test]
fn topological_sort_rejects_cycles() {
    let graph = graph(&[("a", "b"), ("b", "c"), ("c", "a"), ("x", "a")]);
    let error = topological_sort(&graph).unwrap_err().to_string();
    assert!(error.contains("cycle"), "unexpected error: {error}");
}

#[test]
fn topological_sort_rejects_self_loop() {
    let graph = graph(&[("a", "a")]);
    assert!(topological_sort(&graph).is_err());
}

#[test]
fn connected_components_ignore_edge_direction() {
    let graph = graph(&[("a", "b"), ("c", "b"), ("d", "e"), ("f", "f")]);
    assert_eq!(
        connected_components(&graph),
        vec![kws(&["a", "b", "c"]), kws(&["d", "e"]), kws(&["f"])]
    );
}

#[test]
fn connected_components_of_empty_graph_is_empty() {
    let graph = EdgeList::default();
    assert!(connected_components(&graph).is_empty());
    assert_eq!(topological_sort(&graph).unwrap(), Vec::<Value>::new());
}

#[test]
fn minigraf_graph_depth_first() {
    let (graph, node) = load_named_graph(&[("a", "b"), ("b", "c"), ("a", "d")]);
    let order = depth_first(&graph, &node("a"));
    assert_eq!(order.len(), 3);
    // Neighbor order from the database is not fixed, but DFS must visit c right after b.
    let b = order.iter().position(|n| *n == node("b")).unwrap();
    assert_eq!(order[b + 1..].first(), Some(&node("c")));
}

#[test]
fn minigraf_graph_shortest_path() {
    let (graph, node) = load_named_graph(&[("a", "b"), ("b", "c"), ("c", "d"), ("a", "d")]);
    assert_eq!(
        shortest_path(&graph, &node("a"), &node("d")),
        Some(vec![node("a"), node("d")])
    );
    assert_eq!(shortest_path(&graph, &node("d"), &node("a")), None);
}

#[test]
fn minigraf_graph_topological_sort() {
    let edges = [
        ("fetch", "build"),
        ("build", "test"),
        ("build", "package"),
        ("test", "deploy"),
        ("package", "deploy"),
    ];
    let (graph, node) = load_named_graph(&edges);
    let order = topological_sort(&graph).unwrap();
    assert_eq!(order.len(), 5);
    let position = |name: &str| order.iter().position(|n| *n == node(name)).unwrap();
    for (from, to) in edges {
        assert!(
            position(from) < position(to),
            "{from} must come before {to}"
        );
    }

    let (cyclic, _) = load_named_graph(&[("a", "b"), ("b", "a")]);
    assert!(topological_sort(&cyclic).is_err());
}

#[test]
fn minigraf_graph_connected_components() {
    let (graph, node) = load_named_graph(&[("a", "b"), ("c", "b"), ("x", "y")]);
    let mut components: Vec<Vec<Value>> = connected_components(&graph);
    assert_eq!(components.len(), 2);
    components.sort_by_key(|component| std::cmp::Reverse(component.len()));
    assert_eq!(components[0].len(), 3);
    for name in ["a", "b", "c"] {
        assert!(components[0].contains(&node(name)));
    }
    assert_eq!(components[1].len(), 2);
    for name in ["x", "y"] {
        assert!(components[1].contains(&node(name)));
    }
}

fn query_one(db: &Minigraf, query: &str) -> Value {
    let QueryResult::QueryResults { results, .. } = db.execute(query).unwrap() else {
        panic!("expected query results");
    };

    results[0][0].clone()
}

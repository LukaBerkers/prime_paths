use std::{env, fs};

use eyre::{eyre, Result};
use graphviz_rust::{
    dot_structures::{Attribute, Edge, Graph as DotGraph, GraphAttributes, Id, Node, Stmt},
    printer::{DotPrinter, PrinterContext},
};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    match &args[..] {
        [_, file_name] => {
            let contents = fs::read_to_string(file_name)?;
            let graph = graphviz_rust::parse(&contents).map_err(|err| eyre!(err))?;

            let prime_paths = find_prime_paths(graph);

            for path in prime_paths {
                let dot = path.print(&mut PrinterContext::default());
                println!("{}", dot);
            }

            Ok(())
        }
        _ => Err(eyre!("Wrong number of arguments")),
    }
}

#[derive(Copy, Clone, Debug)]
enum GraphType {
    Undirected,
    Directed,
}

#[derive(Clone, Debug)]
struct Graph {
    graph_type: GraphType,
    strict: bool,
    attributes: Vec<Attribute>,
    graph_attributes: Vec<GraphAttributes>,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

fn find_prime_paths(graph: DotGraph) -> Vec<DotGraph> {
    let graph = deconstruct_graph(graph);
    let graph = reconstruct_graph(graph, "g");

    todo!()
}

fn deconstruct_graph(graph: DotGraph) -> Graph {
    let (graph_type, strict, stmts) = match graph {
        DotGraph::Graph { strict, stmts, .. } => (GraphType::Undirected, strict, stmts),
        DotGraph::DiGraph { strict, stmts, .. } => (GraphType::Directed, strict, stmts),
    };

    let mut attributes: Vec<Attribute> = Vec::new();
    let mut graph_attributes: Vec<GraphAttributes> = Vec::new();
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    for stmt in stmts {
        match stmt {
            Stmt::Attribute(a) => attributes.push(a),
            Stmt::GAttribute(gas) => graph_attributes.push(gas),
            Stmt::Node(n) => nodes.push(n),
            Stmt::Edge(e) => edges.push(e),
            Stmt::Subgraph(_) => todo!(),
        }
    }

    Graph {
        graph_type,
        strict,
        attributes,
        graph_attributes,
        nodes,
        edges,
    }
}

fn reconstruct_graph(graph: Graph, id: &str) -> DotGraph {
    let stmt_iter = graph
        .attributes
        .into_iter()
        .map(Stmt::Attribute)
        .chain(graph.graph_attributes.into_iter().map(Stmt::GAttribute))
        .chain(graph.nodes.into_iter().map(Stmt::Node))
        .chain(graph.edges.into_iter().map(Stmt::Edge));
    let stmts = Vec::from_iter(stmt_iter);

    match graph.graph_type {
        GraphType::Undirected => DotGraph::Graph {
            id: Id::Plain(id.to_owned()),
            strict: graph.strict,
            stmts,
        },
        GraphType::Directed => DotGraph::DiGraph {
            id: Id::Plain(id.to_owned()),
            strict: graph.strict,
            stmts,
        },
    }
}

use std::{env, fs};

use eyre::{eyre, Result};
use graphviz_rust::{
    dot_structures::{Attribute, Edge, Graph as DotGraph, GraphAttributes, Node, Stmt},
    printer::{DotPrinter, PrinterContext},
};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    match &args[..] {
        [_, file_name] => {
            let contents = fs::read_to_string(file_name)?;
            let graph = graphviz_rust::parse(&contents).map_err(|err| eyre!(err))?;

            let prime_paths = find_prime_paths(&graph);

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

fn find_prime_paths(graph: &DotGraph) -> Vec<DotGraph> {
    let graph = deconstruct_graph(graph);
    dbg!(graph);

    todo!()
}

fn deconstruct_graph(graph: &DotGraph) -> Graph {
    let (graph_type, strict, stmts) = match graph {
        DotGraph::Graph { strict, stmts, .. } => (GraphType::Undirected, *strict, stmts),
        DotGraph::DiGraph { strict, stmts, .. } => (GraphType::Directed, *strict, stmts),
    };

    let mut attributes: Vec<Attribute> = Vec::new();
    let mut graph_attributes: Vec<GraphAttributes> = Vec::new();
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    for stmt in stmts {
        match stmt {
            Stmt::Attribute(a) => attributes.push(a.clone()),
            Stmt::GAttribute(gas) => graph_attributes.push(gas.clone()),
            Stmt::Node(n) => nodes.push(n.clone()),
            Stmt::Edge(e) => edges.push(e.clone()),
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

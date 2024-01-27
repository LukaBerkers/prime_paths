use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::Path as FsPath,
};

use eyre::{eyre, Result};
use graphviz_rust::{
    dot_structures::{
        Attribute, Edge, EdgeTy, Graph as DotGraph, GraphAttributes, Id, Node, Stmt, Vertex,
    },
    printer::{DotPrinter, PrinterContext},
};

const OUT_DIR: &str = "./paths";

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    match &args[..] {
        [_, file_name] => {
            let contents = fs::read_to_string(file_name)?;
            let graph = graphviz_rust::parse(&contents).map_err(|err| eyre!(err))?;

            let prime_paths = find_prime_paths(graph);

            let out_dir = FsPath::new(OUT_DIR);
            fs::create_dir(out_dir)?;
            for (i, path) in prime_paths.iter().enumerate() {
                let mut file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(out_dir.join(i.to_string()).with_extension("dot"))?;
                let dot = path.print(&mut PrinterContext::default());
                file.write_all(dot.as_bytes())?;
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

impl Graph {
    fn get_node(&self, id: &Id) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id.0 == *id)
    }
}

#[derive(Clone, Debug, Default)]
struct Path {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Path {
    fn clone_extend(&self, node: &Node, edge: &Edge) -> Self {
        let mut extended_path = self.clone();
        extended_path.nodes.push(node.clone());
        extended_path.edges.push(edge.clone());

        extended_path
    }

    fn is_subpath_of(&self, other: &Path) -> bool {
        let len = self.nodes.len();
        let other_len = other.nodes.len();
        if len > other_len {
            return false;
        }

        for i in 0..=(other_len - len) {
            if other.nodes[i..(i + len)] == self.nodes {
                return true;
            }
        }

        false
    }
}

fn find_prime_paths(graph: DotGraph) -> Vec<DotGraph> {
    let graph = deconstruct_graph(graph);
    let mut paths: Vec<Path> = graph
        .nodes
        .iter()
        .map(|n| Path {
            nodes: vec![n.clone()],
            edges: Vec::new(),
        })
        .collect();
    let mut unextendable_paths = Vec::new();

    while !paths.is_empty() {
        paths = extend_paths(&graph, paths, &mut unextendable_paths);
    }

    let prime_paths = remove_subpaths(unextendable_paths);

    prime_paths
        .into_iter()
        .enumerate()
        .map(|(i, p)| {
            let path_graph = Graph {
                nodes: p.nodes,
                edges: p.edges,
                ..graph.clone()
            };

            reconstruct_graph(path_graph, &i.to_string())
        })
        .collect()
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

fn extend_paths(graph: &Graph, paths: Vec<Path>, unextendable_paths: &mut Vec<Path>) -> Vec<Path> {
    let mut extended_paths = Vec::new();
    for path in paths {
        let nexts = get_nexts(graph, &path);

        if nexts.is_empty() {
            unextendable_paths.push(path);
        } else {
            for (node, edge) in nexts {
                if node == path.nodes.first().expect("empty path") {
                    // full cycle
                    unextendable_paths.push(path.clone_extend(node, edge));
                } else if path.nodes.contains(node) {
                    // would form cycle
                    unextendable_paths.push(path.clone());
                } else {
                    extended_paths.push(path.clone_extend(node, edge));
                }
            }
        }
    }

    extended_paths
}

fn get_nexts<'g>(graph: &'g Graph, path: &Path) -> Vec<(&'g Node, &'g Edge)> {
    match path.nodes.last() {
        Some(last) => {
            let neighbors = get_neighbors(graph, last);

            neighbors
                .iter()
                .map(|(id, e)| (graph.get_node(id).expect("unknown node"), *e))
                .collect()
        }
        None => panic!("Empty path"),
    }
}

fn get_neighbors<'g>(graph: &'g Graph, node: &Node) -> Vec<(&'g Id, &'g Edge)> {
    let id = &node.id.0;
    let mut neighbors = Vec::new();

    for edge in &graph.edges {
        match &edge.ty {
            EdgeTy::Pair(Vertex::N(from), Vertex::N(to)) if *id == from.0 => {
                neighbors.push((&to.0, edge))
            }
            EdgeTy::Pair(_, _) => {}
            EdgeTy::Chain(_) => todo!(),
        }
    }

    neighbors
}

fn remove_subpaths(paths: Vec<Path>) -> Vec<Path> {
    let mut maximal_paths = Vec::new();

    for path in paths.into_iter().rev() {
        if !maximal_paths
            .iter()
            .any(|max_path| path.is_subpath_of(max_path))
        {
            maximal_paths.push(path);
        }
    }

    maximal_paths
}

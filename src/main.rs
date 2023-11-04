use std::{env, fs};

use eyre::{eyre, Result};
use graphviz_rust::{
    dot_structures::Graph,
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

fn find_prime_paths(graph: &Graph) -> Vec<Graph> {
    todo!()
}

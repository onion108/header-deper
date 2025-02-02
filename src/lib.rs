use crate::walker::IncludeWalker;

pub mod parser;
pub mod walker;

pub fn display_graph(walker: &IncludeWalker) {
    for (_, dep) in &walker.graph {
        println!("{}: ", dep.file);
        for dep in &dep.dependencies {
            println!("\t{}", walker.graph[dep].file);
        }
        println!()
    }
}


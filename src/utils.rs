use std::{fmt::Debug, fs};

use petgraph::{
    dot::{Config, Dot},
    EdgeType, Graph,
};

pub fn write_dot_to_file<N, E, Ty>(graph: Graph<N, E, Ty>, path: &'static str) -> anyhow::Result<()>
where
    N: Debug,
    E: Debug,
    Ty: EdgeType,
{
    let dot = Dot::with_config(&graph, &[Config::EdgeNoLabel]);
    fs::write(path, format!("{:?}", dot))?;
    Ok(())
}

pub fn write_degree_sequence_to_file(
    degree_sequence: Vec<usize>,
    path: &'static str,
) -> anyhow::Result<()> {
    let line = degree_sequence
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(",");
    fs::write(path, line)?;
    Ok(())
}

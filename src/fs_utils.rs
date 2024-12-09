use std::{fmt::Debug, fs, path::Path};

use petgraph::{
    dot::{Config, Dot},
    EdgeType, Graph,
};

pub fn write_dot_to_file<N, E, Ty, P: AsRef<Path>>(
    graph: Graph<N, E, Ty>,
    path: P,
) -> anyhow::Result<()>
where
    N: Debug,
    E: Debug,
    Ty: EdgeType,
{
    let dot = Dot::with_config(&graph, &[Config::EdgeNoLabel]);
    fs::write(path, format!("{:?}", dot))?;
    Ok(())
}

pub fn write_values_to_file<P: AsRef<Path>>(values: Vec<usize>, path: P) -> anyhow::Result<()> {
    let line = values
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(",");
    fs::write(path, line)?;
    Ok(())
}

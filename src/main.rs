use std::path::Path;

use barabasi_albert_simulation::{
    simulation::{BarabasiAlbertClassic, DegreeSequence, Gen},
    utils::{write_degree_sequence_to_file, write_dot_to_file},
};

fn main() -> anyhow::Result<()> {
    let mut barabasi = BarabasiAlbertClassic::new(10, 5, 10000000);
    let graph = barabasi.generate();
    let graph_degree_sequence = graph.degree_sequence();
    let graph_name = format!(
        "barabasi_classic_n={}_m={}_end={}",
        barabasi.n, barabasi.m, barabasi.end_time
    );
    let base = "resources";

    let mut dot_path = Path::new(base).join("dots").join(graph_name.clone());
    dot_path.set_extension("dot");

    let mut degree_sequence_path = Path::new(base)
        .join("degree_sequences")
        .join(graph_name.clone());
    degree_sequence_path.set_extension("txt");

    println!("{:?}", dot_path);
    println!("{:?}", degree_sequence_path);
    write_degree_sequence_to_file(graph_degree_sequence, degree_sequence_path)?;
    write_dot_to_file(graph, &dot_path)?;
    Ok(())
}

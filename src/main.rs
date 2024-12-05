use std::path::Path;

use barabasi_albert_simulation::{
    simulation::{BarabasiAlbertClassic, Simulation},
    utils::{write_degree_sequence_to_file, write_dot_to_file},
};

fn main() -> anyhow::Result<()> {
    let it_number = 1000;
    let mut barabasi = BarabasiAlbertClassic::new(10, 5, 100000);
    let sim = Simulation {
        iteration_number: it_number,
    };

    let graph_name = format!(
        "barabasi_classic_n={}_m={}_end={}_mean_over={}",
        barabasi.n, barabasi.m, barabasi.end_time, it_number
    );

    let base = "resources";

    let mut dot_path = Path::new(base).join("dots").join(graph_name.clone());
    dot_path.set_extension("dot");

    let mut degree_sequence_path = Path::new(base)
        .join("degree_sequences")
        .join(graph_name.clone());
    degree_sequence_path.set_extension("txt");

    let degree_sequence_mean = sim.simulate();

    write_degree_sequence_to_file(degree_sequence_mean, degree_sequence_path)?;
    Ok(())
}

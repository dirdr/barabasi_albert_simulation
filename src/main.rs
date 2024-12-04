use barabasi_albert_simulation::{
    simulation::{BarabasiAlbertClassic, DegreeSequence, Gen},
    utils::{write_degree_sequence_to_file, write_dot_to_file},
};

fn main() -> anyhow::Result<()> {
    let mut barabasi = BarabasiAlbertClassic::new(10, 5, 1000000);
    let graph = barabasi.generate();
    let graph_degree_sequence = graph.degree_sequence();
    write_degree_sequence_to_file(
        graph_degree_sequence,
        "resources/degree_sequences/full_barabasi.txt",
    )?;
    write_dot_to_file(graph, "resources/dots/full_barabasi.dot")?;
    Ok(())
}

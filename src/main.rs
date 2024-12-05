use std::path::{Path, PathBuf};

use barabasi_albert_simulation::{
    models::{BarabasiAlbertClassic, BarabasiAlbertPetgraphWrapper, GraphType, ModelConfig},
    simulation::Simulation,
    utils::write_degree_sequence_to_file,
};

fn main() -> anyhow::Result<()> {
    simulate_custom()?;
    simulate_builtin()?;
    Ok(())
}

fn simulate_builtin() -> anyhow::Result<()> {
    let builtin = ModelConfig {
        initial_nodes: 10,
        edges_increment: 5,
        end_time: 100000,
        starting_graph_type: GraphType::Complete,
    };
    let sim_builtin = Simulation::new(100);
    let over = sim_builtin.simulate::<BarabasiAlbertPetgraphWrapper>(builtin);
    let custom_path = format!(
        "BA_BUILTIN_n={}_m={}_tmax={}_it={}",
        &builtin.initial_nodes, &builtin.edges_increment, &builtin.end_time, over.iteration_number
    );
    let path = generate_path(custom_path, "degree_sequences", Some("txt"));
    let degree_sequence = over.get_mean_degree_sequence();
    write_degree_sequence_to_file(degree_sequence, path)?;
    Ok(())
}

fn simulate_custom() -> anyhow::Result<()> {
    let custom = ModelConfig {
        initial_nodes: 10,
        edges_increment: 5,
        end_time: 100000,
        starting_graph_type: GraphType::Complete,
    };
    let sim_custom = Simulation::new(100);
    let over = sim_custom.simulate::<BarabasiAlbertClassic>(custom);
    let custom_path = format!(
        "BA_CUSTOM_n={}_m={}_tmax={}_it={}",
        &custom.initial_nodes, &custom.edges_increment, &custom.end_time, over.iteration_number
    );
    let path = generate_path(custom_path, "degree_sequences", Some("txt"));
    let degree_sequence = over.get_mean_degree_sequence();
    write_degree_sequence_to_file(degree_sequence, path)?;
    Ok(())
}

fn generate_path(
    graph_name: String,
    sub_folder: &'static str,
    extension: Option<&'static str>,
) -> PathBuf {
    let mut path = Path::new("resources").join(sub_folder).join(graph_name);
    if let Some(extension) = extension {
        path.set_extension(extension);
    }
    path
}

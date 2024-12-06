use std::path::{Path, PathBuf};

use barabasi_albert_simulation::{
    args::Args,
    models::{BarabasiAlbertClassic, BarabasiAlbertPetgraphWrapper, ModelConfig},
    simulation::Simulation,
    utils::write_values_to_file,
};
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    static TRACKED_VERTICES: &[usize] = &[1, 10, 100, 1000];
    let model_config = ModelConfig::from_args(&args, TRACKED_VERTICES);
    simulate_custom(&model_config, args.iteration_number)?;
    simulate_builtin(&model_config, args.iteration_number)?;
    Ok(())
}

fn simulate_builtin(model_config: &ModelConfig, iteration_number: usize) -> anyhow::Result<()> {
    let sim_builtin = Simulation::new(iteration_number);
    let over = sim_builtin.simulate::<BarabasiAlbertPetgraphWrapper>(*model_config);
    let custom_path = format!(
        "BA_BUILTIN_n={}_m={}_tmax={}_it={}",
        &model_config.initial_nodes,
        &model_config.edges_increment,
        &model_config.end_time,
        over.iteration_number
    );
    let ds_path = generate_path(custom_path.clone(), "degree_sequences", Some("txt"));
    let degree_sequence = over.get_mean_degree_sequence();
    write_values_to_file(degree_sequence, ds_path)?;
    Ok(())
}

fn simulate_custom(model_config: &ModelConfig, iteration_number: usize) -> anyhow::Result<()> {
    let sim_custom = Simulation::new(iteration_number);
    let over = sim_custom.simulate_with_tracking::<BarabasiAlbertClassic>(*model_config);

    let custom_path = format!(
        "BA_CUSTOM_n={}_m={}_tmax={}_it={}",
        &model_config.initial_nodes,
        &model_config.edges_increment,
        &model_config.end_time,
        over.iteration_number
    );

    let path = generate_path(custom_path, "degree_sequences", Some("txt"));

    let vertices_evolution = over.get_vertex_evolution::<BarabasiAlbertClassic>();
    for vertex in model_config.tracked_vertices {
        let name = format!(
            "VERTEX_{}_BA_CUSTOM_n={}_m={}_tmax={}_it={}",
            vertex,
            &model_config.initial_nodes,
            &model_config.edges_increment,
            &model_config.end_time,
            over.iteration_number
        );
        let vertices_evolution_path = generate_path(name, "vertices_evolution", Some("txt"));
        let value = vertices_evolution[&petgraph::graph::NodeIndex::new(*vertex)].clone();

        write_values_to_file(value, vertices_evolution_path)?;
    }
    let degree_sequence = over.get_mean_degree_sequence();
    write_values_to_file(degree_sequence, path)?;
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

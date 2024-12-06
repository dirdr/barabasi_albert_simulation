use std::path::{Path, PathBuf};

use barabasi_albert_simulation::{
    args::{self, Args},
    models::{BarabasiAlbertClassic, BarabasiAlbertPetgraphWrapper, GraphType, ModelConfig},
    simulation::Simulation,
    utils::write_degree_sequence_to_file,
};
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let model_config = ModelConfig {
        initial_nodes: args.n,
        edges_increment: args.m,
        end_time: args.barabasi_end_time,
        starting_graph_type: match args.barabasi_starting_graph_type {
            args::ArgsGraphType::Complete => GraphType::Complete,
            args::ArgsGraphType::Star => GraphType::Star,
        },
    };
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
    let path = generate_path(custom_path, "degree_sequences", Some("txt"));
    let degree_sequence = over.get_mean_degree_sequence();
    write_degree_sequence_to_file(degree_sequence, path)?;
    Ok(())
}

fn simulate_custom(model_config: &ModelConfig, iteration_number: usize) -> anyhow::Result<()> {
    let sim_custom = Simulation::new(iteration_number);
    let over = sim_custom.simulate::<BarabasiAlbertClassic>(*model_config);
    let custom_path = format!(
        "BA_CUSTOM_n={}_m={}_tmax={}_it={}",
        &model_config.initial_nodes,
        &model_config.edges_increment,
        &model_config.end_time,
        over.iteration_number
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

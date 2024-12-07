use std::path::{Path, PathBuf};

use barabasi_albert_simulation::{
    args::{Args, ArgsModelType},
    models::{
        BarabasiAlbertClassic, BarabasiAlbertNoGrowth, BarabasiAlbertPetgraphWrapper,
        BarabasiAlbertRandomAttachement, ModelConfig,
    },
    simulation::Simulation,
    utils::write_values_to_file,
};
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    static TRACKED_VERTICES: &[usize] = &[1, 10, 100, 1000];
    let model_config = ModelConfig::from_args(&args, TRACKED_VERTICES);
    simulate_custom(&model_config, args.iteration_number, args.model)?;
    //simulate_builtin(&model_config, args.iteration_number)?;
    Ok(())
}

fn simulate_custom(
    model_config: &ModelConfig,
    iteration_number: usize,
    model_type: ArgsModelType,
) -> anyhow::Result<()> {
    let sim_custom = Simulation::new(iteration_number);

    let over = match model_type {
        ArgsModelType::GrowthPreferential => {
            sim_custom.simulate_with_tracking::<BarabasiAlbertClassic>(*model_config)
        }
        ArgsModelType::GrowthRandom => {
            sim_custom.simulate_with_tracking::<BarabasiAlbertRandomAttachement>(*model_config)
        }
        ArgsModelType::NoGrowthPreferential => {
            sim_custom.simulate_with_tracking::<BarabasiAlbertNoGrowth>(*model_config)
        }
    };

    let model_name = format!("{}", model_type);

    let custom_path = format!(
        "{}_n={}_m={}_tmax={}_it={}",
        model_name,
        &model_config.initial_nodes,
        &model_config.edges_increment,
        &model_config.end_time,
        over.iteration_number
    );

    let path = generate_path(custom_path, "degree_sequences", Some("txt"));

    let vertices_evolution = over.get_vertex_evolution::<BarabasiAlbertClassic>();
    for vertex in model_config.tracked_vertices {
        let custom_path = format!(
            "{}_vertex={}_n={}_m={}_tmax={}_it={}",
            model_name,
            vertex,
            &model_config.initial_nodes,
            &model_config.edges_increment,
            &model_config.end_time,
            over.iteration_number
        );
        let vertices_evolution_path = generate_path(custom_path, "vertices_evolution", Some("txt"));
        let value = vertices_evolution[&petgraph::graph::NodeIndex::new(*vertex)].clone();

        write_values_to_file(value, vertices_evolution_path)?;
    }

    let degree_sequence = over.get_mean_degree_sequence();
    write_values_to_file(degree_sequence, path)?;
    Ok(())
}

#[allow(dead_code)]
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

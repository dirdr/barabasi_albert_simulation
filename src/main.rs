use std::path::{Path, PathBuf};

use barabasi_albert_simulation::{
    args::{Args, ArgsGraphType, ArgsModelType},
    fs_utils::write_values_to_file,
    models::{
        BarabasiAlbertClassic, BarabasiAlbertNoGrowth, BarabasiAlbertRandomAttachement, ModelConfig,
    },
    simulation::Simulation,
};
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    validate_args(&args)?;

    // TRACKED_ARRIVALS[i] = k mean we track the evolution of the vertex arriving at time 100,
    // in the graph, counting the initial nodes in the graph.
    // For the `BarabasiAlbertNoGrowth` models, the tracked vertex will be the one picked at time i
    static TRACKED_ARRIVALS: &[usize] = &[1, 10, 100, 1000];

    let model_config = ModelConfig::from_args(&args, TRACKED_ARRIVALS);
    simulate_custom(&model_config, args.iterations, args.model)?;
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
            sim_custom.simulate::<BarabasiAlbertClassic>(*model_config)
        }
        ArgsModelType::GrowthRandom => {
            sim_custom.simulate::<BarabasiAlbertRandomAttachement>(*model_config)
        }
        ArgsModelType::NoGrowthPreferential => {
            sim_custom.simulate::<BarabasiAlbertNoGrowth>(*model_config)
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

    let arrival_evolution = over.get_mean_arrival_evolution::<BarabasiAlbertClassic>();
    for vertex in model_config.tracked_arrivals {
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

        if let Some(value) = arrival_evolution.get(vertex) {
            write_values_to_file(value.clone(), vertices_evolution_path)?;
        }
    }

    let degree_sequence = over.get_degree_sequence();
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

fn validate_args(args: &Args) -> anyhow::Result<()> {
    if matches!(
        args.model,
        ArgsModelType::GrowthPreferential | ArgsModelType::GrowthRandom
    ) && args.starting_graph == ArgsGraphType::Disconnected
    {
        Err(anyhow::anyhow!(
            "Starting graph `Disconnected` is not allowed with models `GrowthPreferential` or `GrowthRandom`."
        ))
    } else {
        Ok(())
    }
}

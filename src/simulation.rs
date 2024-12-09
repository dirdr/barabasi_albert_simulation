use std::collections::HashMap;

use petgraph::graph::NodeIndex;

use crate::{
    graph_utils::DegreeSequence,
    models::{FromModelConfig, Gen, ModelConfig, TrackVertices},
};

/// Barabasi-Albert model is random by nature, to have analysis on the models
/// we simulate the results `iteration_number` time with the goal to average our two simulation goal
/// 1. The degree sequence of the network
/// 2. The evolution of degree of the different vertices listed inside `tracked_vertices`, those
///    degree are identified by the time step `i` they arrive in the network.
pub struct Simulation<S: SimulationState> {
    pub iteration_number: usize,
    pub degree_sequence: Option<Vec<usize>>,
    state: S,
}

pub trait SimulationState {}

pub struct Over {
    pub vertices_evolution: Option<HashMap<NodeIndex, Vec<usize>>>,
}

pub struct Start {}

impl SimulationState for Over {}
impl SimulationState for Start {}

impl<S: SimulationState> Simulation<S> {
    pub fn mean_vectors(vectors: &[Vec<usize>]) -> Vec<usize> {
        assert!(!vectors.is_empty(), "Input vector list cannot be empty");

        let num_vectors = vectors.len();
        let vector_length = vectors[0].len();

        assert!(
            vectors.iter().all(|v| v.len() == vector_length),
            "All vectors must have the same length"
        );

        (0..vector_length)
            .map(|i| {
                let sum: usize = vectors.iter().map(|v| v[i]).sum();
                (sum as f64 / num_vectors as f64).ceil() as usize
            })
            .collect()
    }
}

impl Simulation<Start> {
    pub fn new(iteration_number: usize) -> Self {
        Self {
            iteration_number,
            degree_sequence: None,
            state: Start {},
        }
    }

    pub fn simulate<G: FromModelConfig + Gen>(self, model_config: ModelConfig) -> Simulation<Over> {
        let mut sequence = None;
        for _ in 0..self.iteration_number {
            let mut model: G = FromModelConfig::from_model_config(model_config);
            let graph = model.generate();
            if sequence.is_none() {
                sequence = Some(graph.degree_sequence());
            }
        }
        Simulation {
            degree_sequence: sequence,
            iteration_number: self.iteration_number,
            state: Over {
                vertices_evolution: None,
            },
        }
    }

    pub fn simulate_with_tracking<G: FromModelConfig + Gen + TrackVertices>(
        self,
        model_config: ModelConfig,
    ) -> Simulation<Over> {
        let mut sequence = None;
        let mut vertices_evolution: HashMap<NodeIndex, Vec<Vec<usize>>> = HashMap::new();

        for _ in 0..self.iteration_number {
            let mut model: G = FromModelConfig::from_model_config(model_config);
            let graph = model.generate();
            for vid in model_config.tracked_timesteps {
                vertices_evolution
                    .entry(NodeIndex::new(*vid))
                    .or_default()
                    .push(model.get_vertex_evolution(NodeIndex::new(*vid)))
            }
            if sequence.is_none() {
                sequence = Some(graph.degree_sequence());
            }
        }
        let meaned_vertices_evolution: HashMap<NodeIndex, Vec<usize>> = vertices_evolution
            .into_iter()
            .map(|(k, ce)| (k, Simulation::<Start>::mean_vectors(&ce)))
            .collect();

        for k in meaned_vertices_evolution.keys() {
            println!(
                "Vertex : {:?}, vertices evolution len {:?}",
                k,
                meaned_vertices_evolution[k].len()
            );
        }

        Simulation {
            degree_sequence: sequence,
            iteration_number: self.iteration_number,
            state: Over {
                vertices_evolution: Some(meaned_vertices_evolution),
            },
        }
    }
}

impl Simulation<Over> {
    pub fn get_mean_degree_sequence(&self) -> Vec<usize> {
        if let Some(ds) = &self.degree_sequence {
            return ds.clone();
        }
        unreachable!("Type state pattern prevent degree sequence being None")
    }

    pub fn get_vertex_evolution<G: TrackVertices>(&self) -> HashMap<NodeIndex, Vec<usize>> {
        if let Some(ve) = &self.state.vertices_evolution {
            return ve.clone();
        }
        unreachable!("Type state pattern prevent vertex evolution from being None")
    }
}

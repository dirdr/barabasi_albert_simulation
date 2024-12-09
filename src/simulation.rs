use std::collections::HashMap;

use crate::{
    graph_utils::DegreeSequence,
    models::{FromModelConfig, Gen, ModelConfig, VerticesEvolutionMarker},
    vertices_evolution::TrackVertices,
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
    pub arrival_evolution: Option<HashMap<usize, Vec<usize>>>,
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

    pub fn simulate<M: FromModelConfig + Gen + VerticesEvolutionMarker>(
        self,
        model_config: ModelConfig,
    ) -> Simulation<Over> {
        let mut sequence = None;
        let mut all_arrival_evolution: HashMap<usize, Vec<Vec<usize>>> = HashMap::new();

        for _ in 0..self.iteration_number {
            let mut model: M = FromModelConfig::from_model_config(model_config);
            let vertices_evolution = model.vertices_evolution();

            let graph = model.generate();

            for arrival in model_config.tracked_arrivals {
                let arrival_evolution = vertices_evolution.get_arrival_evolution(arrival);
                if arrival_evolution.is_none() {
                    panic!("Cannot retreive arrival evolution");
                }

                all_arrival_evolution
                    .entry(*arrival)
                    .or_default()
                    .push(arrival_evolution.unwrap());
            }

            if sequence.is_none() {
                sequence = Some(graph.degree_sequence());
            }
        }

        let meaned_arrivals_evolution: HashMap<usize, Vec<usize>> = all_arrival_evolution
            .into_iter()
            .map(|(k, ce)| (k, Simulation::<Start>::mean_vectors(&ce)))
            .collect();

        for k in meaned_arrivals_evolution.keys() {
            println!(
                "Vertex : {:?}, vertices evolution len {:?}",
                k,
                meaned_arrivals_evolution[k].len()
            );
        }

        Simulation {
            degree_sequence: sequence,
            iteration_number: self.iteration_number,
            state: Over {
                arrival_evolution: Some(meaned_arrivals_evolution),
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

    pub fn get_arrival_evolution<G: VerticesEvolutionMarker>(&self) -> HashMap<usize, Vec<usize>> {
        if let Some(ve) = &self.state.arrival_evolution {
            return ve.clone();
        }
        unreachable!("Type state pattern prevent vertex evolution from being None")
    }
}

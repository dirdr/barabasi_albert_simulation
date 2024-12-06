use std::collections::HashMap;

use petgraph::graph::{NodeIndex, UnGraph};
use petgraph_gen::{barabasi_albert_graph, complete_graph, star_graph};
use rand::{distributions::Uniform, prelude::Distribution, thread_rng, Rng};

use crate::args::{Args, ArgsGraphType};

/// A Model that is capable of itself from a `ModelConfig`
pub trait FromModelConfig {
    fn from_model_config(model_config: ModelConfig) -> Self;
}

/// A Mode that is capable to generate into a graph
pub trait Gen {
    fn generate(&mut self) -> UnGraph<(), ()>;
}

/// A Model that is capable of stepping into the simulation
pub trait Step<R> {
    fn step(&mut self, rng: &mut R);
}

/// A Model that is able to compute it's degree sequence
pub trait DegreeSequence {
    fn degree_sequence(&self) -> Vec<usize>;
}

/// A Model that is able to track the evolution of a vertex into the simulation
pub trait TrackVertices {
    fn get_vertex_evolution(&self, vertex_id: NodeIndex) -> Vec<usize>;
    fn update_vertices_evolution(&mut self);
}

/// A graph that is able to give a name
pub trait Name {
    fn get_name(&self) -> String;
}

#[derive(Debug, Copy, Clone)]
pub enum GraphType {
    Complete,
    Star,
}

/// Represent the starting parameters of a Barabasi-Albert model.
#[derive(Debug, Copy, Clone)]
pub struct ModelConfig {
    pub initial_nodes: usize,
    pub edges_increment: usize,
    pub end_time: usize,
    pub starting_graph_type: GraphType,
    pub tracked_vertices: &'static [usize],
}

impl ModelConfig {
    pub fn from_args(args: &Args, tracked_vertices: &'static [usize]) -> Self {
        Self {
            initial_nodes: args.n,
            edges_increment: args.m,
            end_time: args.barabasi_end_time,
            starting_graph_type: match args.barabasi_starting_graph_type {
                ArgsGraphType::Complete => GraphType::Complete,
                ArgsGraphType::Star => GraphType::Star,
            },
            tracked_vertices,
        }
    }
}

/// A Barabasi-Albert model with vertex growth and preferential attachement
pub struct BarabasiAlbertClassic {
    pub model_config: ModelConfig,
    graph: UnGraph<(), ()>,
    stubs: Vec<NodeIndex>,
    vertices_evolution: HashMap<NodeIndex, Vec<usize>>,
}

/// A Barabasi-Albert model with preferential attachement but without vertex growth.
pub struct BarabasiAlbertNoGrowth;

/// A Barabasi-Albert model with vertex growth and random attachement
/// Each time a vertex is added, it attach to `m` random vertices in the network.
pub struct BarabasiAlbertRandomAttachement;

/// A petgraph wrapper, for comparison purpose with our custom implementation of the
/// Barabasi-Albert model.
pub struct BarabasiAlbertPetgraphWrapper {
    pub model_config: ModelConfig,
}

impl FromModelConfig for BarabasiAlbertPetgraphWrapper {
    fn from_model_config(model_config: ModelConfig) -> Self {
        Self { model_config }
    }
}

impl Gen for BarabasiAlbertPetgraphWrapper {
    fn generate(&mut self) -> UnGraph<(), ()> {
        let mut rng = thread_rng();
        // n in petgraph-gen is the number of final node in the graph, so `initial_nodes` +
        // `end_time`
        let n = self.model_config.initial_nodes + self.model_config.end_time;
        let initial_graph: UnGraph<(), ()> = match self.model_config.starting_graph_type {
            GraphType::Complete => complete_graph(self.model_config.initial_nodes),
            // `star_graph` generate graph with n - 1 nodes
            GraphType::Star => star_graph(self.model_config.initial_nodes - 1),
        };
        barabasi_albert_graph(
            &mut rng,
            n,
            self.model_config.edges_increment,
            Some(initial_graph),
        )
    }
}

impl<R> Step<R> for BarabasiAlbertClassic
where
    R: Rng + Sized,
{
    fn step(&mut self, rng: &mut R) {
        let new_node = self.graph.add_node(());
        let uniform = Uniform::new(0, self.stubs.len());
        let mut picked = vec![false; self.model_config.initial_nodes + self.model_config.end_time];
        let mut targets = vec![NodeIndex::new(0); self.model_config.edges_increment];
        let mut i = 0;
        while i < self.model_config.edges_increment {
            let random_index = uniform.sample(rng);
            let target = self.stubs[random_index];
            // To prevent multi-edge
            if !picked[target.index()] {
                picked[target.index()] = true;
                targets[i] = target;
                i += 1;
            }
        }
        for &target in &targets {
            self.graph.add_edge(new_node, target, ());
            self.stubs.push(new_node);
            self.stubs.push(target);
            picked[target.index()] = false;
        }
    }
}

impl FromModelConfig for BarabasiAlbertClassic {
    fn from_model_config(model_config: ModelConfig) -> Self {
        assert!(
            model_config.initial_nodes >= 1,
            "The number of initial vertices must be greater than 1"
        );

        assert!(
            model_config.edges_increment <= model_config.initial_nodes,
            "The number of initial node need to be greater than number of new connexion per step"
        );

        assert!(
            model_config.edges_increment >= 1,
            "The number of new connexion per step must be greater than one"
        );

        let graph: UnGraph<(), ()> = match model_config.starting_graph_type {
            GraphType::Complete => complete_graph(model_config.initial_nodes),
            GraphType::Star => star_graph(model_config.initial_nodes - 1),
        };

        let mut stubs = vec![];
        for node in graph.node_indices() {
            for _ in graph.edges(node) {
                stubs.push(node);
            }
        }

        Self {
            model_config,
            graph,
            stubs,
            vertices_evolution: HashMap::new(),
        }
    }
}

impl Gen for BarabasiAlbertClassic {
    /// Generate a Barabasi Albert graph with `n` initial nodes and `m` newly created edges at each
    /// time step
    fn generate(&mut self) -> UnGraph<(), ()> {
        let mut rng = thread_rng();
        for _ in 0..self.model_config.end_time {
            self.step(&mut rng);
            self.update_vertices_evolution();
        }
        self.graph.clone()
    }
}

impl TrackVertices for BarabasiAlbertClassic {
    fn get_vertex_evolution(&self, vertex_id: NodeIndex) -> Vec<usize> {
        let default = Vec::new();
        self.vertices_evolution
            .get(&vertex_id)
            .unwrap_or(&default)
            .clone()
    }

    fn update_vertices_evolution(&mut self) {
        for vertex in self.model_config.tracked_vertices {
            let node_index = NodeIndex::new(*vertex);
            self.vertices_evolution
                .entry(node_index)
                .or_default()
                .push(self.graph.neighbors(node_index).count())
        }
    }
}

impl<N, E> DegreeSequence for UnGraph<N, E> {
    fn degree_sequence(&self) -> Vec<usize> {
        let mut out = vec![];
        for node in self.node_indices() {
            let num = self.edges(node).count();
            out.push(num);
        }
        out
    }
}

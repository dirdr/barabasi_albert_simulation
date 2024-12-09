use core::panic;
use std::collections::HashMap;

use petgraph::graph::{NodeIndex, UnGraph};
use petgraph_gen::{complete_graph, star_graph};
use rand::{distributions::Uniform, prelude::Distribution, thread_rng, Rng};

use crate::args::{Args, ArgsGraphType};

pub trait Complete {
    fn is_complete(&self) -> bool;
}

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
    fn step(&mut self, rng: &mut R) -> bool;
}

/// A Model that is able to compute it's degree sequence
pub trait DegreeSequence {
    fn degree_sequence(&self) -> Vec<usize>;
}

/// A Model that is able to track the evolution of a vertex into the simulation
pub trait TrackVertices {
    fn get_vertex_evolution(&self, vertex_id: NodeIndex) -> Vec<usize>;
    fn update_vertices_evolution(&mut self, time: usize);
}

/// A graph that is able to give a name
pub trait Name {
    fn get_name(&self) -> String;
}

#[derive(Debug, Copy, Clone)]
pub enum GraphType {
    Complete,
    Star,
    Disconnected,
}

/// Represent the starting parameters of a Barabasi-Albert model.
#[derive(Debug, Copy, Clone)]
pub struct ModelConfig {
    pub initial_nodes: usize,
    /// Number of new edges per time step of the simulation
    pub edges_increment: usize,
    /// Number of iterations of the simulation
    pub end_time: usize,
    pub starting_graph_type: GraphType,
    // Times `t` at which we start tracking a vertex evolution,
    // The vertex is either the one added at time `t` for `BarabasiAlbertClassic` and `BarabasiAlbertRandomAttachement`,
    // or the node connected at time `t` for `BarabasiAlbertNoGrowth`.
    pub tracked_vertices: &'static [usize],
}

/// A Barabasi-Albert model with vertex growth and preferential attachement
pub struct BarabasiAlbertClassic {
    pub model_config: ModelConfig,
    graph: UnGraph<(), ()>,
    stubs: Vec<NodeIndex>,
    picked: Vec<bool>,
    targets: Vec<NodeIndex>,
    // TODO refactor pour prendre une strcture commune qui isole ce comportement
    vertices_evolution: HashMap<NodeIndex, Vec<usize>>,
}

/// A Barabasi-Albert model with vertex growth and random attachement
pub struct BarabasiAlbertRandomAttachement {
    pub model_config: ModelConfig,
    graph: UnGraph<(), ()>,
    // To avoid calling `graph.node_indices().count()` which is O(n)
    node_count: usize,
    picked: Vec<bool>,
    targets: Vec<NodeIndex>,
    // TODO refactor pour prendre une strcture commune qui isole ce comportement
    // Refactor cela pour que cela soit le strcture qui implémente le trait et que je n'ai aps
    // besoin de implementé en obucle pour tous les trusc
    vertices_evolution: HashMap<NodeIndex, Vec<usize>>,
}

/// A Barabasi-Albert model with preferential attachement but without vertex growth.
pub struct BarabasiAlbertNoGrowth {
    model_config: ModelConfig,
    graph: UnGraph<(), ()>,
    stubs: Vec<NodeIndex>,
    picked: Vec<bool>,
    targets: Vec<NodeIndex>,
    tracked_vertices: Vec<NodeIndex>,
    vertices_evolution: HashMap<NodeIndex, Vec<usize>>,
    initial_uniform: Uniform<usize>,
    current_time_step: usize,
}

impl ModelConfig {
    pub fn from_args(args: &Args, tracked_vertices: &'static [usize]) -> Self {
        assert!(
            args.n >= 1,
            "The number of initial vertices must be greater than 1"
        );

        assert!(
            args.m <= args.n,
            "The number of initial node need to be greater than number of new connexion per step"
        );

        assert!(
            args.m >= 1,
            "The number of new connexion per step must be greater than one"
        );

        Self {
            initial_nodes: args.n,
            edges_increment: args.m,
            end_time: args.t_max,
            starting_graph_type: match args.starting_graph {
                ArgsGraphType::Complete => GraphType::Complete,
                ArgsGraphType::Star => GraphType::Star,
                ArgsGraphType::Disconnected => GraphType::Disconnected,
            },
            tracked_vertices,
        }
    }
}

impl FromModelConfig for BarabasiAlbertClassic {
    fn from_model_config(model_config: ModelConfig) -> Self {
        let graph: UnGraph<(), ()> = match model_config.starting_graph_type {
            GraphType::Complete => complete_graph(model_config.initial_nodes),
            GraphType::Star => star_graph(model_config.initial_nodes - 1),
            GraphType::Disconnected => {
                panic!("This initial graph type is only for barabasi-abert no growth")
            }
        };

        let mut stubs = vec![];
        for node in graph.node_indices() {
            for _ in graph.edges(node) {
                stubs.push(node);
            }
        }

        let picked = vec![false; model_config.initial_nodes + model_config.end_time];
        let targets = vec![NodeIndex::new(0); model_config.edges_increment];

        Self {
            model_config,
            graph,
            stubs,
            picked,
            targets,
            vertices_evolution: HashMap::new(),
        }
    }
}

impl FromModelConfig for BarabasiAlbertRandomAttachement {
    fn from_model_config(model_config: ModelConfig) -> Self {
        let graph: UnGraph<(), ()> = match model_config.starting_graph_type {
            GraphType::Complete => complete_graph(model_config.initial_nodes),
            GraphType::Star => star_graph(model_config.initial_nodes - 1),
            GraphType::Disconnected => {
                panic!("This initial graph type is only for barabasi-abert no growth")
            }
        };

        let picked = vec![false; model_config.initial_nodes + model_config.end_time];
        let targets = vec![NodeIndex::new(0); model_config.edges_increment];

        Self {
            model_config,
            graph,
            picked,
            targets,
            vertices_evolution: HashMap::new(),
            node_count: model_config.initial_nodes,
        }
    }
}

impl FromModelConfig for BarabasiAlbertNoGrowth {
    fn from_model_config(model_config: ModelConfig) -> Self {
        let mut graph: UnGraph<(), ()> = match model_config.starting_graph_type {
            GraphType::Complete => complete_graph(model_config.initial_nodes),
            GraphType::Star => star_graph(model_config.initial_nodes - 1),
            GraphType::Disconnected => UnGraph::<(), ()>::new_undirected(),
        };

        let mut stubs = vec![];

        if let GraphType::Disconnected = model_config.starting_graph_type {
            for _ in 0..model_config.initial_nodes {
                let node = graph.add_node(());
                stubs.push(node);
            }
        } else {
            for node in graph.node_indices() {
                // TODO fix ce truc horrible
                for _ in graph.edges(node) {
                    stubs.push(node);
                }
            }
        }

        let picked = vec![false; model_config.initial_nodes + model_config.end_time];
        let targets = vec![NodeIndex::new(0); model_config.edges_increment];

        Self {
            model_config,
            graph,
            stubs,
            picked,
            targets,
            vertices_evolution: HashMap::new(),
            initial_uniform: Uniform::new(0, model_config.initial_nodes),
            tracked_vertices: vec![],
            current_time_step: 0,
        }
    }
}

impl<R> Step<R> for BarabasiAlbertClassic
where
    R: Rng + Sized,
{
    fn step(&mut self, rng: &mut R) -> bool {
        let new_node = self.graph.add_node(());
        let uniform = Uniform::new(0, self.stubs.len());
        let mut i = 0;
        while i < self.model_config.edges_increment {
            let random_index = uniform.sample(rng);
            let target = self.stubs[random_index];
            // To prevent multi-edge
            if !self.picked[target.index()] {
                self.picked[target.index()] = true;
                self.targets[i] = target;
                i += 1;
            }
        }

        for &target in &self.targets {
            self.graph.add_edge(new_node, target, ());
            self.stubs.push(new_node);
            self.stubs.push(target);
            self.picked[target.index()] = false;
        }

        true
    }
}

impl<R> Step<R> for BarabasiAlbertRandomAttachement
where
    R: Rng + Sized,
{
    fn step(&mut self, rng: &mut R) -> bool {
        let new_node = self.graph.add_node(());
        let uniform = Uniform::new(0, self.node_count);
        let mut i = 0;
        while i < self.model_config.edges_increment {
            let random_index = uniform.sample(rng);
            // To prevent multi-edge
            if !self.picked[random_index] {
                self.picked[random_index] = true;
                self.targets[i] = NodeIndex::new(random_index);
                i += 1;
            }
        }

        for &target in &self.targets {
            self.graph.add_edge(new_node, target, ());
            self.picked[target.index()] = false;
        }

        self.node_count += 1;
        true
    }
}

impl<R> Step<R> for BarabasiAlbertNoGrowth
where
    R: Rng + Sized,
{
    fn step(&mut self, rng: &mut R) -> bool {
        // Explicit check of complete graph to avoid adding duplicate edges
        if self.graph.is_complete() {
            return false;
        }
        let stubs_uniform = Uniform::new(0, self.stubs.len());
        let random_node = NodeIndex::new(self.initial_uniform.sample(rng));
        // Add the node that have been picked at time step i to the list of tracked vertex
        if self
            .model_config
            .tracked_vertices
            .contains(&self.current_time_step)
        {
            self.tracked_vertices.push(random_node);
        }
        let mut i = 0;
        while i < self.model_config.edges_increment {
            let random_index = stubs_uniform.sample(rng);
            let target = self.stubs[random_index];
            // To prevent multi-edge
            if target != random_node
                && !self.picked[target.index()]
                && self.graph.find_edge(random_node, target).is_none()
            {
                self.picked[target.index()] = true;
                self.targets[i] = target;
                i += 1;
            }
        }

        for &target in &self.targets {
            self.graph.add_edge(random_node, target, ());
            self.stubs.push(target);
            self.stubs.push(random_node);
            self.picked[target.index()] = false;
        }

        true
    }
}

// TODO une fois que j'ai fait toutes les implementations et que tout marche
// Regarder si je ne peux pas foutre tout cela dans une blanket implementation
impl Gen for BarabasiAlbertClassic {
    fn generate(&mut self) -> UnGraph<(), ()> {
        let mut rng = thread_rng();
        for time in 1..=self.model_config.end_time {
            if !self.step(&mut rng) {
                break;
            }
            self.update_vertices_evolution(time);
        }
        self.graph.clone()
    }
}

impl Gen for BarabasiAlbertRandomAttachement {
    fn generate(&mut self) -> UnGraph<(), ()> {
        let mut rng = thread_rng();
        for time in 1..=self.model_config.end_time {
            if !self.step(&mut rng) {
                break;
            }
            self.update_vertices_evolution(time);
        }
        self.graph.clone()
    }
}

impl Gen for BarabasiAlbertNoGrowth {
    fn generate(&mut self) -> UnGraph<(), ()> {
        let mut rng = thread_rng();
        for time in 1..=self.model_config.end_time {
            if !self.step(&mut rng) {
                break;
            }
            self.update_vertices_evolution(time);
        }
        self.graph.clone()
    }
}

// TODO une fois que j'ai fait toutes les implementations et que tout marche
// Regarder si je ne peux pas foutre tout cela dans une blanket implementation
// REGARDER AUSSI si je peux pas wrapper la logique de suivre les tracked vertex dans une structure
// de donnée custom
impl TrackVertices for BarabasiAlbertClassic {
    fn get_vertex_evolution(&self, vertex_id: NodeIndex) -> Vec<usize> {
        let default = Vec::new();
        self.vertices_evolution
            .get(&vertex_id)
            .unwrap_or(&default)
            .clone()
    }

    fn update_vertices_evolution(&mut self, time: usize) {
        for vertex in self.model_config.tracked_vertices {
            let node_index = NodeIndex::new(*vertex);
            // Only start updating the node degree evolution if we are at least at time step where
            // he arrive
            if *vertex > time {
                continue;
            }
            self.vertices_evolution
                .entry(node_index)
                .or_default()
                .push(self.graph.neighbors(node_index).count())
        }
    }
}

impl TrackVertices for BarabasiAlbertRandomAttachement {
    fn get_vertex_evolution(&self, vertex_id: NodeIndex) -> Vec<usize> {
        let default = Vec::new();
        self.vertices_evolution
            .get(&vertex_id)
            .unwrap_or(&default)
            .clone()
    }

    fn update_vertices_evolution(&mut self, time: usize) {
        for vertex in self.model_config.tracked_vertices {
            let node_index = NodeIndex::new(*vertex);
            // Only start updating the node degree evolution if we are at least at time step where
            // he arrive
            if *vertex > time {
                continue;
            }
            self.vertices_evolution
                .entry(node_index)
                .or_default()
                .push(self.graph.neighbors(node_index).count())
        }
    }
}

impl TrackVertices for BarabasiAlbertNoGrowth {
    // TODO mettre une fonction get vertex evolution all pour pouvoir tout gather
    fn get_vertex_evolution(&self, vertex_id: NodeIndex) -> Vec<usize> {
        let default = Vec::new();
        self.vertices_evolution
            .get(&vertex_id)
            .unwrap_or(&default)
            .clone()
    }

    /// This implementation is different because this time, we don't track the node that we add at
    /// time i but rather the node that has been choosen to be connected at time `time`, which can
    /// or not be the `time` vertex.
    fn update_vertices_evolution(&mut self, _: usize) {
        for vertex in &self.tracked_vertices {
            let node_index = vertex;
            self.vertices_evolution
                .entry(*node_index)
                .or_default()
                .push(self.graph.neighbors(*node_index).count())
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

impl<N, E> Complete for UnGraph<N, E> {
    fn is_complete(&self) -> bool {
        let num_nodes = self.node_count();
        let expected_edges = (num_nodes * (num_nodes - 1)) / 2;
        self.edge_count() == expected_edges
    }
}

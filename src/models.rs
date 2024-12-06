use petgraph::graph::{NodeIndex, UnGraph};
use petgraph_gen::{barabasi_albert_graph, complete_graph, star_graph};
use rand::{thread_rng, Rng};

pub trait FromModelConfig {
    fn from_model_config(model_config: ModelConfig) -> Self;
}

/// A Barabasi-Albert variant that is capable to generate until `t_max`
pub trait Gen {
    fn generate(&mut self) -> UnGraph<(), ()>;
}

/// A Barabasi-Albert that is capable of stepping into the simulation
pub trait Step<R> {
    fn step(&mut self, rng: &mut R);
}

/// A Graph that is able to compute it's degree sequence
pub trait DegreeSequence {
    fn degree_sequence(&self) -> Vec<usize>;
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
}

pub struct BarabasiAlbertClassic {
    graph: UnGraph<(), ()>,
    stubs: Vec<NodeIndex>,
    pub initial_nodes: usize,
    pub edges_increment: usize,
    pub end_time: usize,
}

pub struct BarabasiAlbertNoGrowth;
pub struct BarabasiAlbertRandomAttachement;

/// This wrapper is just for comparison purpose with our custom implementation of the
/// Barabasi-Albert model.
pub struct BarabasiAlbertPetgraphWrapper {
    pub initial_nodes: usize,
    pub edges_increment: usize,
    pub end_time: usize,
    pub initial_graph_type: GraphType,
}

impl FromModelConfig for BarabasiAlbertPetgraphWrapper {
    fn from_model_config(model_config: ModelConfig) -> Self {
        Self {
            initial_nodes: model_config.initial_nodes,
            edges_increment: model_config.edges_increment,
            end_time: model_config.end_time,
            initial_graph_type: model_config.starting_graph_type,
        }
    }
}

impl Gen for BarabasiAlbertPetgraphWrapper {
    fn generate(&mut self) -> UnGraph<(), ()> {
        let mut rng = thread_rng();
        // n in petgraph-gen is the number of final node in the graph, so `initial_nodes` +
        // `end_time`
        let n = self.initial_nodes + self.end_time;
        let initial_graph: UnGraph<(), ()> = match self.initial_graph_type {
            GraphType::Complete => complete_graph(self.initial_nodes),
            // `star_graph` generate graph with n - 1 nodes
            GraphType::Star => star_graph(self.initial_nodes - 1),
        };
        barabasi_albert_graph(&mut rng, n, self.edges_increment, Some(initial_graph))
    }
}

impl<R> Step<R> for BarabasiAlbertClassic
where
    R: Rng + Sized,
{
    fn step(&mut self, rng: &mut R) {
        let new_node = self.graph.add_node(());
        let mut targets = vec![];
        while targets.len() < self.edges_increment {
            let random_index = rng.gen_range(0..self.stubs.len());
            let target = self.stubs[random_index];
            // To prevent multi-edge
            if !targets.contains(&target) {
                targets.push(target);
            }
        }
        for &target in &targets {
            self.graph.add_edge(new_node, target, ());
            self.stubs.push(new_node);
            self.stubs.push(target);
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
            graph,
            stubs,
            initial_nodes: model_config.initial_nodes,
            edges_increment: model_config.edges_increment,
            end_time: model_config.end_time,
        }
    }
}

impl Gen for BarabasiAlbertClassic {
    /// Generate a Barabasi Albert graph with `n` initial nodes and `m` newly created edges at each
    /// time step
    fn generate(&mut self) -> UnGraph<(), ()> {
        let mut rng = thread_rng();
        for _ in 0..self.end_time {
            self.step(&mut rng);
        }
        self.graph.clone()
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

use petgraph::graph::{NodeIndex, UnGraph};
use petgraph_gen::{complete_graph, star_graph};
use rand::{distributions::Uniform, prelude::Distribution, thread_rng, Rng};

use crate::{
    args::{Args, ArgsGraphType},
    graph_utils::Complete,
    vertices_evolution::{TrackVertices, VerticesEvolution},
};

/// A Model that can be created from a `ModelConfig`
pub trait FromModelConfig {
    fn from_model_config(model_config: ModelConfig) -> Self;
}

/// A Mode that is capable to generate into a graph
pub trait Gen {
    fn generate(&mut self) -> UnGraph<(), ()>;
}

/// A Model that is capable of stepping into the simulation
pub trait Step<R> {
    fn step(&mut self, rng: &mut R, time: usize) -> bool;
}

/// A Model that is capable of handing it's vertices evolutions
pub trait VerticesEvolutionMarker {
    fn vertices_evolution(&self) -> &VerticesEvolution;
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
    pub tracked_arrivals: &'static [usize],
}

/// A Barabasi-Albert model with vertex growth and preferential attachement
pub struct BarabasiAlbertClassic {
    pub vertices_evolution: VerticesEvolution,
    model_config: ModelConfig,
    graph: UnGraph<(), ()>,
    stubs: Vec<NodeIndex>,
    picked: Vec<bool>,
    targets: Vec<NodeIndex>,
}

/// A Barabasi-Albert model with vertex growth and random attachement
pub struct BarabasiAlbertRandomAttachement {
    pub vertices_evolution: VerticesEvolution,
    model_config: ModelConfig,
    graph: UnGraph<(), ()>,
    // To avoid calling `graph.node_indices().count()` which is O(n)
    picked: Vec<bool>,
    targets: Vec<NodeIndex>,
    node_count: usize,
}

/// A Barabasi-Albert model with preferential attachement but without vertex growth.
pub struct BarabasiAlbertNoGrowth {
    pub vertices_evolution: VerticesEvolution,
    model_config: ModelConfig,
    graph: UnGraph<(), ()>,
    stubs: Vec<NodeIndex>,
    picked: Vec<bool>,
    targets: Vec<NodeIndex>,
    initial_uniform: Uniform<usize>,
}

impl ModelConfig {
    pub fn from_args(args: &Args, tracked_arrivals: &'static [usize]) -> Self {
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
            tracked_arrivals,
        }
    }
}

impl FromModelConfig for BarabasiAlbertClassic {
    fn from_model_config(model_config: ModelConfig) -> Self {
        let graph = model_config
            .starting_graph_type
            .to_graph(model_config.initial_nodes);

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
            vertices_evolution: VerticesEvolution::new(),
        }
    }
}

impl FromModelConfig for BarabasiAlbertRandomAttachement {
    fn from_model_config(model_config: ModelConfig) -> Self {
        let graph = model_config
            .starting_graph_type
            .to_graph(model_config.initial_nodes);

        let picked = vec![false; model_config.initial_nodes + model_config.end_time];
        let targets = vec![NodeIndex::new(0); model_config.edges_increment];

        Self {
            model_config,
            graph,
            picked,
            targets,
            vertices_evolution: VerticesEvolution::new(),
            node_count: model_config.initial_nodes,
        }
    }
}

impl FromModelConfig for BarabasiAlbertNoGrowth {
    fn from_model_config(model_config: ModelConfig) -> Self {
        let mut stubs = vec![];

        let graph = model_config
            .starting_graph_type
            .to_graph(model_config.initial_nodes);

        for node in graph.node_indices() {
            if let GraphType::Disconnected = model_config.starting_graph_type {
                stubs.push(node);
            } else {
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
            vertices_evolution: VerticesEvolution::new(),
            initial_uniform: Uniform::new(0, model_config.initial_nodes),
        }
    }
}

impl<R> Step<R> for BarabasiAlbertClassic
where
    R: Rng + Sized,
{
    fn step(&mut self, rng: &mut R, time: usize) -> bool {
        let new_node = self.graph.add_node(());
        if self.model_config.tracked_arrivals.contains(&time) {
            self.vertices_evolution.track(time, &new_node);
        }

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
    fn step(&mut self, rng: &mut R, time: usize) -> bool {
        let new_node = self.graph.add_node(());
        if self.model_config.tracked_arrivals.contains(&time) {
            self.vertices_evolution.track(time, &new_node);
        }

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
    fn step(&mut self, rng: &mut R, time: usize) -> bool {
        // Explicit check of complete graph to avoid adding duplicate edges
        if self.graph.is_complete() {
            return false;
        }
        let stubs_uniform = Uniform::new(0, self.stubs.len());
        let mut random_node = NodeIndex::new(self.initial_uniform.sample(rng));

        // Find a non-tracked node (There is a small chance that the random node for arrival `time` has already been picked
        if self.model_config.tracked_arrivals.contains(&time) {
            while !self.vertices_evolution.already_track(&random_node) {
                random_node = NodeIndex::new(self.initial_uniform.sample(rng));
                self.vertices_evolution.track(time, &random_node);
            }
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
// Ou alors plus simple dans un wrapper struct Generate!
impl Gen for BarabasiAlbertClassic {
    fn generate(&mut self) -> UnGraph<(), ()> {
        let mut rng = thread_rng();
        for time in 1..=self.model_config.end_time {
            if !self.step(&mut rng, time) {
                break;
            }
            self.vertices_evolution.update(&self.graph);
        }
        self.graph.clone()
    }
}

impl Gen for BarabasiAlbertRandomAttachement {
    fn generate(&mut self) -> UnGraph<(), ()> {
        let mut rng = thread_rng();
        for time in 1..=self.model_config.end_time {
            if !self.step(&mut rng, time) {
                break;
            }
            self.vertices_evolution.update(&self.graph);
        }
        self.graph.clone()
    }
}

impl Gen for BarabasiAlbertNoGrowth {
    fn generate(&mut self) -> UnGraph<(), ()> {
        let mut rng = thread_rng();
        for time in 1..=self.model_config.end_time {
            if !self.step(&mut rng, time) {
                break;
            }
            self.vertices_evolution.update(&self.graph);
        }
        self.graph.clone()
    }
}

impl VerticesEvolutionMarker for BarabasiAlbertClassic {
    fn vertices_evolution(&self) -> &VerticesEvolution {
        &self.vertices_evolution
    }
}

impl VerticesEvolutionMarker for BarabasiAlbertRandomAttachement {
    fn vertices_evolution(&self) -> &VerticesEvolution {
        &self.vertices_evolution
    }
}

impl VerticesEvolutionMarker for BarabasiAlbertNoGrowth {
    fn vertices_evolution(&self) -> &VerticesEvolution {
        &self.vertices_evolution
    }
}

impl GraphType {
    fn to_graph(self, initial_nodes: usize) -> UnGraph<(), ()> {
        match self {
            GraphType::Complete => complete_graph(initial_nodes),
            GraphType::Star => star_graph(initial_nodes - 1),
            GraphType::Disconnected => {
                let mut g = UnGraph::<(), ()>::new_undirected();
                for _ in 0..initial_nodes {
                    g.add_node(());
                }
                g
            }
        }
    }
}

#[cfg(test)]
mod test {
    use petgraph::visit::EdgeRef;

    use crate::models::{BarabasiAlbertClassic, FromModelConfig, Gen, GraphType, ModelConfig};

    const CONFIG: ModelConfig = ModelConfig {
        initial_nodes: 5,
        edges_increment: 3,
        end_time: 10,
        starting_graph_type: GraphType::Complete,
        tracked_arrivals: &[],
    };

    #[test]
    fn test_barabasi_classic_node_count() {
        let mut model: BarabasiAlbertClassic = FromModelConfig::from_model_config(CONFIG);
        let graph = model.generate();

        // Total nodes = Initial nodes + nodes added at each time step
        assert_eq!(
            graph.node_count(),
            model.model_config.initial_nodes + model.model_config.end_time
        );
    }

    #[test]
    fn test_barabasi_classic_edge_count() {
        let mut model: BarabasiAlbertClassic = FromModelConfig::from_model_config(CONFIG);
        let graph = model.generate();

        // Initial edges = (n * (n - 1)) / 2 for a fully connected graph
        let initial_edges = (CONFIG.initial_nodes * (CONFIG.initial_nodes - 1)) / 2;
        let expected_edges = initial_edges + (CONFIG.edges_increment * CONFIG.end_time);

        assert_eq!(graph.edge_count(), expected_edges);
    }

    #[test]
    fn test_barabasi_classic_no_multi_edges() {
        let mut model: BarabasiAlbertClassic = FromModelConfig::from_model_config(CONFIG);
        let graph = model.generate();

        for node in graph.node_indices() {
            let mut neighbors = vec![];
            for edge in graph.edges(node) {
                let target = edge.target();
                assert!(!neighbors.contains(&target), "Multi-edge detected!");
                neighbors.push(target);
            }
        }
    }

    #[test]
    fn test_barabasi_classic_graph_connectivity() {
        let mut model: BarabasiAlbertClassic = FromModelConfig::from_model_config(CONFIG);
        let graph = model.generate();

        let connected_components = petgraph::algo::connected_components(&graph);
        assert_eq!(connected_components, 1, "Graph is not connected");
    }
}

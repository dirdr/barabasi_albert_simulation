use petgraph::graph::UnGraph;

use crate::models::{FromModelConfig, ModelConfig};

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

/// Barabasi-Albert model is random by nature, to have analysis on the models
/// we simulate the results `iteration_number` time with the goal to average our two simulation goal
/// 1. The degree sequence of the network
/// 2. The evolution of degree of the different vertices listed inside `tracked_vertices`, those
///    degree are identified by the time step `i` they arrive in the network.
pub struct Simulation<S: SimulationState> {
    pub iteration_number: usize,
    pub tracked_vertices: Vec<usize>,
    pub degree_sequence: Option<Vec<usize>>,
    marker: std::marker::PhantomData<S>,
}

pub trait SimulationState {}

pub enum Over {}
pub enum Start {}

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
            marker: std::marker::PhantomData,
            tracked_vertices: vec![],
        }
    }

    pub fn simulate<G: FromModelConfig + Gen>(self, model_config: ModelConfig) -> Simulation<Over> {
        let mut sequences = vec![];
        for _ in 0..self.iteration_number {
            let mut model: G = FromModelConfig::from_model_config(model_config);
            let graph = model.generate();
            sequences.push(graph.degree_sequence());
        }
        let mean = Simulation::<Start>::mean_vectors(&sequences);
        Simulation {
            degree_sequence: Some(mean),
            marker: std::marker::PhantomData,
            iteration_number: self.iteration_number,
            tracked_vertices: self.tracked_vertices,
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

#[cfg(test)]
mod tests {

    use petgraph::visit::EdgeRef;

    use crate::{
        models::{BarabasiAlbertClassic, FromModelConfig, GraphType, ModelConfig},
        simulation::{DegreeSequence, Gen},
    };

    #[test]
    fn test_barabasi_classic_node_count() {
        let config = ModelConfig {
            initial_nodes: 5,
            edges_increment: 3,
            end_time: 10,
            starting_graph_type: GraphType::Complete,
        };
        let mut model: BarabasiAlbertClassic = FromModelConfig::from_model_config(config);
        let graph = model.generate();

        // Total nodes = Initial nodes + nodes added at each time step
        assert_eq!(graph.node_count(), model.initial_nodes + model.end_time);
    }

    #[test]
    fn test_barabasi_classic_edge_count() {
        let config = ModelConfig {
            initial_nodes: 5,
            edges_increment: 3,
            end_time: 10,
            starting_graph_type: GraphType::Complete,
        };
        let mut model: BarabasiAlbertClassic = FromModelConfig::from_model_config(config);
        let graph = model.generate();

        // Initial edges = (n * (n - 1)) / 2 for a fully connected graph
        let initial_edges = (config.initial_nodes * (config.initial_nodes - 1)) / 2;
        let expected_edges = initial_edges + (config.edges_increment * config.end_time);

        assert_eq!(graph.edge_count(), expected_edges);
    }

    #[test]
    fn test_barabasi_classic_no_multi_edges() {
        let config = ModelConfig {
            initial_nodes: 5,
            edges_increment: 3,
            end_time: 10,
            starting_graph_type: GraphType::Complete,
        };
        let mut model: BarabasiAlbertClassic = FromModelConfig::from_model_config(config);
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
        let config = ModelConfig {
            initial_nodes: 5,
            edges_increment: 3,
            end_time: 10,
            starting_graph_type: GraphType::Complete,
        };
        let mut model: BarabasiAlbertClassic = FromModelConfig::from_model_config(config);
        let graph = model.generate();

        let connected_components = petgraph::algo::connected_components(&graph);
        assert_eq!(connected_components, 1, "Graph is not connected");
    }

    #[test]
    fn test_degree_sequence_empty_graph() {
        use petgraph::graph::UnGraph;

        let graph: UnGraph<(), ()> = UnGraph::new_undirected();
        let degree_seq = graph.degree_sequence();

        assert!(
            degree_seq.is_empty(),
            "Degree sequence of an empty graph should be empty"
        );
    }

    #[test]
    fn test_degree_sequence_single_node() {
        use petgraph::graph::UnGraph;

        let mut graph: UnGraph<(), ()> = UnGraph::new_undirected();
        graph.add_node(());

        let degree_seq = graph.degree_sequence();

        assert_eq!(
            degree_seq,
            vec![0],
            "Single node graph should have a degree sequence of [0]"
        );
    }

    #[test]
    fn test_degree_sequence_fully_connected() {
        use petgraph::graph::UnGraph;

        let mut graph: UnGraph<(), ()> = UnGraph::new_undirected();
        let nodes: Vec<_> = (0..5).map(|_| graph.add_node(())).collect();

        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                graph.add_edge(nodes[i], nodes[j], ());
            }
        }

        let degree_seq = graph.degree_sequence();
        assert_eq!(
            degree_seq,
            vec![4, 4, 4, 4, 4],
            "Fully connected graph with 5 nodes should have degree sequence [4, 4, 4, 4, 4]"
        );
    }

    #[test]
    fn test_degree_sequence_path_graph() {
        use petgraph::graph::UnGraph;

        let mut graph: UnGraph<(), ()> = UnGraph::new_undirected();
        let nodes: Vec<_> = (0..5).map(|_| graph.add_node(())).collect();

        for i in 0..(nodes.len() - 1) {
            graph.add_edge(nodes[i], nodes[i + 1], ());
        }

        let degree_seq = graph.degree_sequence();
        assert_eq!(
            degree_seq,
            vec![1, 2, 2, 2, 1],
            "Path graph with 5 nodes should have degree sequence [1, 2, 2, 2, 1]"
        );
    }

    #[test]
    fn test_degree_sequence_star_graph() {
        use petgraph::graph::UnGraph;

        let mut graph: UnGraph<(), ()> = UnGraph::new_undirected();
        let center = graph.add_node(());

        for _ in 0..4 {
            let peripheral = graph.add_node(());
            graph.add_edge(center, peripheral, ());
        }

        let degree_seq = graph.degree_sequence();
        let mut expected = vec![4];
        expected.extend(vec![1; 4]);

        assert_eq!(
            degree_seq, expected,
            "Star graph with 5 nodes should have degree sequence [4, 1, 1, 1, 1]"
        );
    }

    #[test]
    fn test_degree_sequence_with_isolated_nodes() {
        use petgraph::graph::UnGraph;

        let mut graph: UnGraph<(), ()> = UnGraph::new_undirected();
        let _ = graph.add_node(());
        let _ = graph.add_node(());

        let connected1 = graph.add_node(());
        let connected2 = graph.add_node(());

        graph.add_edge(connected1, connected2, ());

        let degree_seq = graph.degree_sequence();
        let expected = vec![0, 0, 1, 1];
        assert_eq!(
            degree_seq, expected,
            "Graph with isolated nodes should include zeros in the degree sequence"
        );
    }

    #[test]
    fn test_degree_sequence_sum_equals_twice_edges() {
        use petgraph::graph::UnGraph;

        let mut graph: UnGraph<(), ()> = UnGraph::new_undirected();
        let nodes: Vec<_> = (0..4).map(|_| graph.add_node(())).collect();

        graph.add_edge(nodes[0], nodes[1], ());
        graph.add_edge(nodes[1], nodes[2], ());
        graph.add_edge(nodes[2], nodes[3], ());
        graph.add_edge(nodes[3], nodes[0], ());
        graph.add_edge(nodes[0], nodes[2], ());

        let degree_seq = graph.degree_sequence();
        let degree_sum: usize = degree_seq.iter().sum();

        assert_eq!(
            degree_sum,
            2 * graph.edge_count(),
            "Sum of degree sequence should be twice the number of edges"
        );
    }
}

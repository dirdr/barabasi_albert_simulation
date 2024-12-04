use petgraph::graph::NodeIndex;
use petgraph::graph::UnGraph;
use petgraph_gen::complete_graph;
use rand::thread_rng;
use rand::Rng;

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

pub struct BarabasiAlbertClassic {
    graph: UnGraph<(), ()>,
    stubs: Vec<NodeIndex>,
    pub n: usize,
    pub m: usize,
    pub end_time: usize,
}

pub struct BarabasiAlbertNoGrowth;
pub struct BarabasiAlbertRandomAttachement;

impl<R> Step<R> for BarabasiAlbertClassic
where
    R: Rng + Sized,
{
    fn step(&mut self, rng: &mut R) {
        let new_node = self.graph.add_node(());
        let mut targets = vec![];
        while targets.len() < self.m {
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

impl BarabasiAlbertClassic {
    pub fn new(n: usize, m: usize, end_time: usize) -> Self {
        let graph = complete_graph(n);
        let mut stubs = vec![];
        for node in graph.node_indices() {
            for _ in graph.edges(node) {
                stubs.push(node);
            }
        }
        Self {
            graph,
            stubs,
            n,
            m,
            end_time,
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

#[cfg(test)]
mod tests {
    use petgraph::visit::EdgeRef;

    use crate::simulation::{BarabasiAlbertClassic, DegreeSequence, Gen};

    #[test]
    fn test_barabasi_classic_node_count() {
        let mut gen = BarabasiAlbertClassic::new(5, 3, 10);
        let graph = gen.generate();

        // Total nodes = Initial nodes + nodes added at each time step
        assert_eq!(graph.node_count(), gen.n + gen.end_time);
    }

    #[test]
    fn test_barabasi_classic_edge_count() {
        let mut gen = BarabasiAlbertClassic::new(5, 3, 10);
        let graph = gen.generate();

        // Initial edges = (n * (n - 1)) / 2 for a fully connected graph
        let initial_edges = (gen.n * (gen.n - 1)) / 2;
        let expected_edges = initial_edges + (gen.m * gen.end_time);

        assert_eq!(graph.edge_count(), expected_edges);
    }

    #[test]
    fn test_barabasi_classic_no_multi_edges() {
        let mut gen = BarabasiAlbertClassic::new(5, 3, 10);
        let graph = gen.generate();

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
        let mut gen = BarabasiAlbertClassic::new(5, 3, 10);
        let graph = gen.generate();

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

use petgraph::graph::UnGraph;

/// A Model that is able to compute it's degree sequence
pub trait DegreeSequence {
    fn degree_sequence(&self) -> Vec<usize>;
}

pub trait Complete {
    fn is_complete(&self) -> bool;
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

#[cfg(test)]
pub mod test {
    use crate::graph_utils::DegreeSequence;

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

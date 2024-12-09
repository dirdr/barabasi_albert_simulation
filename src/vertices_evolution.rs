use std::collections::HashMap;

use petgraph::graph::{NodeIndex, UnGraph};

pub trait TrackVertices {
    fn get_arrival_evolution(&self, arrival_time: &usize) -> Option<Vec<usize>>;
    fn update(&mut self, graph: &UnGraph<(), ()>, time: usize);
}

#[derive(Clone)]
pub struct VerticesEvolution {
    /// List of effective tracked vertices index
    tracked_vertices: Vec<NodeIndex>,
    vertices_evolution: HashMap<NodeIndex, Vec<usize>>,
    /// Map the arrival time in the graph with the corresponding NodeIndex in the graph
    arrival_map: HashMap<usize, NodeIndex>,
}

impl VerticesEvolution {
    pub fn new() -> Self {
        Self {
            tracked_vertices: vec![],
            vertices_evolution: HashMap::new(),
            arrival_map: HashMap::new(),
        }
    }

    pub fn track_vertex(&mut self, arrival: usize, vertex: NodeIndex) {
        self.tracked_vertices.push(vertex);
        self.arrival_map.insert(arrival, vertex);
    }
}

impl Default for VerticesEvolution {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackVertices for VerticesEvolution {
    fn get_arrival_evolution(&self, arrival_time: &usize) -> Option<Vec<usize>> {
        if !self.arrival_map.contains_key(arrival_time) {
            return None;
        }
        let node = self.arrival_map.get(arrival_time).unwrap();
        self.vertices_evolution.get(node).cloned()
    }

    fn update(&mut self, graph: &UnGraph<(), ()>, time: usize) {
        for vertex in &self.tracked_vertices {
            // Only start updating the node degree evolution if we are at least at time step where
            // he arrive
            //VERIFIER CELA POUR LE MODELE NO GROWTH
            if vertex.index() > time {
                continue;
            }
            self.vertices_evolution
                .entry(*vertex)
                .or_default()
                .push(graph.neighbors(*vertex).count())
        }
    }
}

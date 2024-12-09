use std::collections::HashMap;

use petgraph::graph::{NodeIndex, UnGraph};

pub trait TrackVertices {
    fn get_arrival_evolution(&self, arrival_time: &usize) -> Option<Vec<usize>>;
    fn update(&mut self, graph: &UnGraph<(), ()>);
}

#[derive(Clone)]
pub struct VerticesEvolution {
    /// List of effective tracked vertices index
    /// The struct will update vertices in this list so they should not be added
    /// Before but rather during the simulation runtime
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

    pub fn track(&mut self, arrival: usize, vertex: NodeIndex) {
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

    fn update(&mut self, graph: &UnGraph<(), ()>) {
        for vertex in &self.tracked_vertices {
            self.vertices_evolution
                .entry(*vertex)
                .or_default()
                .push(graph.neighbors(*vertex).count())
        }
    }
}

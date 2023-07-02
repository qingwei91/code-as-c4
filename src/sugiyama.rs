use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{
    EdgeRef, IntoEdges, IntoNodeIdentifiers, IntoNodeReferences, NodeCount, NodeIndexable,
};
use petgraph::Direction;
use std::cmp::max;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Index;

#[derive(Debug, Clone)]
pub struct LayeredGraph<'a, T, E> {
    layers: Vec<BTreeMap<usize, NodeIndex>>,
    node_to_layer: HashMap<NodeIndex, usize>,
    graph: &'a DiGraph<T, E>,
}

impl<'a, T, E> LayeredGraph<'a, T, E> {
    fn new(graph: &'a DiGraph<T, E>) -> Self {
        let layers = Vec::new();
        let node_to_layer = HashMap::new();
        LayeredGraph {
            layers,
            node_to_layer,
            graph,
        }
    }

    fn add_node_to_layer(&mut self, node: NodeIndex, layer: usize) {
        self.node_to_layer.insert(node, layer);
        if layer >= self.layers.len() {
            self.layers.resize(layer + 1, BTreeMap::new());
        }
        let max_pos = self.layers[layer]
            .keys()
            .max()
            .map(|x| x + 1)
            .unwrap_or(0 as usize);
        self.layers[layer].insert(max_pos, node);
    }

    fn crossing_minimization(&mut self) {
        /*
        We use the median method:
        For each layer,
            for each node,
                compute no of edge for each node connected to previous node
                Then allocate current node n to the median position irt to all connected nodes from previous layer
        */
        for layer_n in 1..self.layers.len() {
            let prev = self.layers.get(layer_n - 1).unwrap();
            let curr = self.layers.get(layer_n).unwrap();
            // we assume we cant change prev to make problem tractable

            let mut new_pos = BTreeMap::<usize, NodeIndex>::new();
            for (pos, node) in curr.iter() {
                let all_neighbors: HashSet<NodeIndex> = self
                    .graph
                    .neighbors_directed(*node, Direction::Incoming)
                    .collect();

                let mut prev_idx: Vec<usize> = prev
                    .iter()
                    .filter_map(|(pos, nid)| {
                        if all_neighbors.contains(nid) {
                            Some(*pos)
                        } else {
                            None
                        }
                    })
                    .collect();

                let mut est_pos = Self::compute_median(&mut prev_idx) as usize;
                while new_pos.contains_key(&est_pos) {
                    est_pos = est_pos + 1;
                }
                new_pos.insert(est_pos, *node);
            }
            self.layers[layer_n] = new_pos;
        }
    }

    fn compute_median(numbers: &mut Vec<usize>) -> f64 {
        numbers.sort(); // Step 1: Sort the vector

        let len = numbers.len();
        let middle_index = len / 2;

        if len % 2 == 0 {
            // Step 4: Compute average of middle two values
            let middle_value = (numbers[middle_index] + numbers[middle_index - 1]) as f64;
            middle_value / 2.0
        } else {
            // Step 3: Return value at middle index
            numbers[middle_index] as f64
        }
    }
}

pub fn sugiyama_method<'a, T, E>(graph: &'a mut DiGraph<T, E>) -> LayeredGraph<'a, T, E>
where
    T: Clone,
    E: Clone,
{
    let mut internal = graph.clone();
    let mut layered_graph = LayeredGraph::new(graph);

    let useless_binding = graph.clone();
    // Step 1: Cycle Removal
    // Feedback Arc Set can be removed to create acyclic graph, or we can revert them to achieve the same effect.
    let feedback_arc_set = petgraph::algo::greedy_feedback_arc_set(&useless_binding);
    feedback_arc_set.for_each(|e| {
        internal.remove_edge(e.id());
        internal.add_edge(e.target(), e.source(), e.weight().clone());
    });

    // Step 2: Layer Assignment (incl hidden nodes)
    let layer_assignment = petgraph::algo::toposort(&internal, None).unwrap();

    for node_id in layer_assignment.into_iter() {
        let incoming_neighbors: Vec<NodeIndex> = internal
            .neighbors_directed(node_id, Direction::Incoming)
            .collect();
        let n_count = incoming_neighbors.len();
        if n_count == 0 {
            layered_graph.add_node_to_layer(node_id, 0);
        } else {
            let max_layer = incoming_neighbors
                .iter()
                .map(|n| layered_graph.node_to_layer.get(&n).unwrap())
                .max()
                .unwrap()
                + 1;
            // inefficient double loop :shrug:
            for neighbor in incoming_neighbors.iter() {
                let neighbor_l = layered_graph.node_to_layer.get(neighbor).unwrap();
                if neighbor_l + 1 < max_layer {
                    for i in (neighbor_l + 1)..max_layer {
                        let hidden_node = NodeIndex::new(usize::MAX);
                        layered_graph.add_node_to_layer(hidden_node, i);
                    }
                }
            }

            layered_graph.add_node_to_layer(node_id, max_layer);
        }
    }

    // Step 3: Crossing Minimization
    layered_graph.crossing_minimization();

    // Steps 4 and 5 are not implemented in this example.

    layered_graph
}

use std::fs;
use crate::parse::parse_to_graph;

#[test]
fn test_sugiyama() {
    let contents = fs::read_to_string("src/example.rustpeg").expect("File not found.");

    let mut graph = parse_to_graph(contents.as_str()).unwrap();

    println!("{:?}", graph);

    let layered = sugiyama_method(&mut graph);
    println!("{:?}", layered)
}

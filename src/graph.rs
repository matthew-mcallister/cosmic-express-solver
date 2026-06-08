use crate::{Content, Grid};
use crate::const_vec::ConstVec;

pub type NodeId = u16;
pub type EdgeId = u16;

#[derive(Debug)]
pub struct Node {
    pub content: Content,
    pub neighbors: ConstVec<NodeId, 4>,
    // Edge data, for future use
    pub out_edges: ConstVec<EdgeId, 4>,
    pub in_edges: ConstVec<EdgeId, 4>,
}

#[derive(Debug)]
pub struct Edge {
    pub src: NodeId,
    pub dst: NodeId,
}

#[derive(Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub entry: NodeId,
    pub num_cars: usize,
}

impl Graph {
    pub fn from_grid(grid: &Grid) -> Self {
        let w = grid.width as usize;
        let h = grid.height as usize;

        let mut cell_to_node: Vec<Option<NodeId>> = vec![None; w * h];
        let mut nodes: Vec<Node> = Vec::new();
        let mut edges: Vec<Edge> = Vec::new();

        for i in 0..h {
            for j in 0..w {
                if grid.get(i as u8, j as u8) == Content::Obstacle {
                    continue;
                }
                let id = nodes.len() as NodeId;
                cell_to_node[i * w + j] = Some(id);
                nodes.push(Node {
                    content: grid.get(i as u8, j as u8),
                    neighbors: ConstVec::new(),
                    out_edges: ConstVec::new(),
                    in_edges: ConstVec::new(),
                });
            }
        }

        for i in 0..h {
            for j in 0..w {
                let Some(src) = cell_to_node[i * w + j] else { continue };

                if j + 1 < w {
                    if let Some(dst) = cell_to_node[i * w + j + 1] {
                        let e1 = edges.len() as EdgeId;
                        edges.push(Edge { src, dst });
                        nodes[src as usize].neighbors.push(dst);
                        nodes[src as usize].out_edges.push(e1);
                        nodes[dst as usize].in_edges.push(e1);

                        let e2 = edges.len() as EdgeId;
                        edges.push(Edge { src: dst, dst: src });
                        nodes[dst as usize].neighbors.push(src);
                        nodes[dst as usize].out_edges.push(e2);
                        nodes[src as usize].in_edges.push(e2);
                    }
                }

                if i + 1 < h {
                    if let Some(dst) = cell_to_node[(i + 1) * w + j] {
                        let e1 = edges.len() as EdgeId;
                        edges.push(Edge { src, dst });
                        nodes[src as usize].neighbors.push(dst);
                        nodes[src as usize].out_edges.push(e1);
                        nodes[dst as usize].in_edges.push(e1);

                        let e2 = edges.len() as EdgeId;
                        edges.push(Edge { src: dst, dst: src });
                        nodes[dst as usize].neighbors.push(src);
                        nodes[dst as usize].out_edges.push(e2);
                        nodes[src as usize].in_edges.push(e2);
                    }
                }
            }
        }

        let entry_idx = grid.entry.0 as usize * w + grid.entry.1 as usize;
        let entry = cell_to_node[entry_idx].expect("entry cell must not be an obstacle");

        Graph { nodes, edges, entry, num_cars: grid.num_cars as usize }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Content, parse_grid};
    use crate::graph::Graph;

    #[test]
    fn test_from_grid_basic() {
        let grid = parse_grid(r"
            e.x
            .B.
        ");
        let graph = Graph::from_grid(&grid);

        assert_eq!(graph.num_cars, 1);
        // 6 non-obstacle cells
        assert_eq!(graph.nodes.len(), 6);
        // 7 undirected adjacencies = 14 directed edges
        assert_eq!(graph.edges.len(), 14);
        // entry at (0,0) = first node
        assert_eq!(graph.entry, 0);
        assert_eq!(graph.nodes[0].content, Content::None);
        assert_eq!(graph.nodes[2].content, Content::Exit);

        // node 0 (0,0) neighbors: right (0,1) and down (1,0)
        assert_eq!(graph.nodes[0].neighbors.len(), 2);
        assert!(graph.nodes[0].neighbors.contains(&1));
        assert!(graph.nodes[0].neighbors.contains(&3));

        // node 4 (1,1) neighbors: up, left, right (no down, at bottom edge)
        assert_eq!(graph.nodes[4].neighbors.len(), 3);
    }

    #[test]
    fn test_from_grid_obstacles_skipped() {
        let grid = parse_grid(r"
            e/
            .x
        ");
        let graph = Graph::from_grid(&grid);
        // obstacle at (0,1) is skipped, so 3 nodes
        assert_eq!(graph.nodes.len(), 3);
        // adjacency: (0,0)-(1,0) and (1,0)-(1,1) = 2 undirected = 4 directed
        assert_eq!(graph.edges.len(), 4);
        // node 0 neighbors: only down (1,0), not right (obstacle)
        assert_eq!(graph.nodes[0].neighbors.len(), 1);
    }
}

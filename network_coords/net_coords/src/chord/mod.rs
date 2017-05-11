extern crate petgraph;

pub mod ids_chain;
pub mod index_id;

use self::petgraph::graphmap::NodeTrait;
use network::{Network};
use chord::index_id::{IndexId};

type RingKey = u64; // A key in the chord ring
type NodeChain = Vec<RingKey>;
type NeighborConnector = Vec<NodeChain>;

// Size of keyspace is 2^L:
const L: usize = 42;


pub struct ChordFingers {
    left: NodeChain<>, 
    right_positive: Vec<NodeChain>,
    right_negative: Vec<NodeChain>,
    // Connectors for neighbors:
    neighbor_connectors: Vec<NeighborConnector>,

    right_randomized: Vec<NodeChain>,
    // Additional random nodes from the keyspace:
    rand_nodes: Vec<NodeChain>, 
}



/// Calculate ring distance from x to y clockwise
fn vdist(xk:RingKey, yk: RingKey) -> RingKey {
    (yk.wrapping_sub(xk)) % 2_u64.pow(L as u32)
}


fn extract_chains<'a> (fingers: &'a ChordFingers) -> 
    Vec<&'a NodeChain> {

    let mut res: Vec<&NodeChain> = Vec::new();
    res.push(&fingers.left);
    {
        let mut push_chains = |chains: &'a Vec<NodeChain>| {
            for chain in chains {
                res.push(chain);
            }
        };
        push_chains(&fingers.right_positive);
        push_chains(&fingers.right_negative);
        push_chains(&fingers.right_randomized);
        for conn in &fingers.neighbor_connectors {
            push_chains(&conn)
        }
    }

    res
}

/// Pass over a chain of node ids. Remove cycles of node ids.
fn remove_cycles(chain: &NodeChain) {
}



/// Perform one fingers iteration for node x: 
/// Take all chains from neighbors and update own chains to the best found chains.
fn iter_fingers<Node: NodeTrait>(x_i: usize, net: Network<Node>, 
             index_id: &IndexId, fingers: &mut Vec<ChordFingers>) {

    let x_id: RingKey = index_id.index_to_id(x_i).unwrap();

    // Collect all chains to one vector. 
    let mut all_chains: Vec<NodeChain> = Vec::new();

    // Add trivial chain (x):
    all_chains.push(vec![x_id]);

    // Add trivial chains (x,nei) where nei is any neighbor of x:
    for neighbor_index in net.igraph.neighbors(x_i) {
        all_chains.push(vec![index_id.index_to_id(neighbor_index).unwrap(), x_id])
    }

    // Add all current chains:
    all_chains.extend(
        extract_chains(&fingers[x_i]).iter().map(|&chain| chain.clone())
    );

    // Update left finger:
    fingers[x_i].left = all_chains.iter().min_by_key(|c| (vdist(c[0], x_id), c.len()) ).unwrap().clone();

    // Find the chain that is closest to target_id from the right.
    let best_right_chain = |target_id| all_chains.iter().min_by_key(|c| 
                                         (vdist(target_id, c[0]), c.len()) ).unwrap().clone();

    // Update all right fingers:
    for i in 0 .. L {
        fingers[x_i].right_positive[i] = best_right_chain((x_id + 2_u64.pow(i as u32)) % 2_u64.pow(L as u32));
    }
    for i in 0 .. L {
        fingers[x_i].right_negative[i] = best_right_chain((x_id - 2_u64.pow(i as u32)) % 2_u64.pow(L as u32));
    }

    // Update neighbor connectors.
    // For determinism, we sort the neighbors before iterating.
    // TODO: Finish here.
    /*
    for (neighbor_vec_index, neighbor_index) in net.igraph.neighbors(x_i).collect::<Vec<_>>().inplace_sort().iter().enumerate() {
        let neighbor_id: RingKey = index_id.index_to_id(neighbor_index).unwrap();

        for cur_id in ids_chain(x_id, neighbor_id) {
            fingers[x_i].neighbor_connectors[neighbor_vec_index]

        }

    }
    */

    // For every maintained chain: Find the best chain.
    //  - Closest to wanted target.
    //  - Shortest possible.
    //      - Eliminate cycles?
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_d() {
        assert!(vdist(1u64,2) == 1);
        assert!(vdist(1u64,101) == 100);
        assert!(vdist(2_u64.pow(L as u32) - 1,1) == 2);
        assert!(vdist(2_u64.pow(L as u32) - 1,0) == 1);
        assert!(vdist(1,0) == 2_u64.pow(L as u32) - 1);
    }
}

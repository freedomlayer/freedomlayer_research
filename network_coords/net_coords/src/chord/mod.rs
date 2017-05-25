extern crate petgraph;
extern crate rand;

pub mod ids_chain;
pub mod semi_routes_array;
pub mod node_fingers;

use std::collections::{HashSet, HashMap, VecDeque};

use self::rand::{Rng, StdRng};
use self::rand::distributions::{IndependentSample, Range};

use network::{Network};
use self::ids_chain::{ids_chain};
use self::semi_routes_array::{SemiRoutesArray, sroute_final_id};
use self::node_fingers::{Finger, NodeFingers, SemiChain};
use index_pair::{index_pair, Pair};


pub type RingKey = u64; // A key in the chord ring
pub type NodeChain = Vec<RingKey>;
pub type NeighborConnector = Vec<NodeChain>;

type RouteField = Vec<HashMap<RingKey,SemiChain>>;
type SemiRoute = Vec<SemiChain>;


/// Calculate ring distance from x to y clockwise
fn vdist(xk:RingKey, yk: RingKey, l: usize) -> RingKey {
    (yk.wrapping_sub(xk)) % 2_u64.pow(l as u32)
}

/// Add cyclic (x + diff) % max_key
fn add_cyc(x: RingKey, diff: i64, l: usize) -> RingKey {
    let max_key = 2_u64.pow(l as u32);
    (if diff >= 0 {
        x.wrapping_add(diff as u64)
    } else {
        x.wrapping_sub((-diff) as u64)
    }) % max_key
}


/// Generate a vector of maintained left target_ids for node with id x_id.
fn gen_left_target_ids(x_id: RingKey, l: usize) -> Vec<RingKey> {
    vec![add_cyc(x_id,-1,l)]
}

/// Generate a vector of maintained right target_ids for node with id x_id.
fn gen_right_target_ids<R: Rng>(x_id: RingKey, net: &Network<RingKey>, 
                                l: usize, mut rng: &mut R) -> Vec<RingKey> {

    let mut target_ids: Vec<RingKey> = Vec::new();

    // Basic right fingers:
    for i in 0 .. l {
        let pow_val = 2_i64.pow(i as u32);
        target_ids.push(add_cyc(x_id,pow_val,l));
        target_ids.push(add_cyc(x_id,-pow_val,l));
    }

    // Neighbor connectors:
    let x_i = net.node_to_index(&x_id).unwrap();
    for neighbor_index in net.igraph.neighbors(x_i) {
        let neighbor_id: RingKey = net.index_to_node(neighbor_index).unwrap().clone();
        for cur_id in ids_chain(x_id, neighbor_id) {
            target_ids.push(cur_id);
        }
    }

    // Right randomized fingers:
    for i in 0 .. l {
        // Randomize a finger value in [2^i, 2^(i+1))
        let rand_range: Range<RingKey> = 
            Range::new(2_u64.pow(i as u32),2_u64.pow((i + 1) as u32));
        let rand_id = rand_range.ind_sample(&mut rng);
        target_ids.push(rand_id);
    }

    // Update random fingers:
    for _ in 0 .. l {
        // Randomize a finger value in [0, 2^l). Completely random in the ring key space.
        let rand_range: Range<RingKey> = Range::new(0u64,2_u64.pow(l as u32));
        let rand_id = rand_range.ind_sample(&mut rng);
        target_ids.push(rand_id);
    }
    
    target_ids
}

/// Initialize maintained fingers for node with index x_i.
fn create_node_fingers<R: Rng>(x_i: usize, net: &Network<RingKey>, 
             l: usize, mut rng: &mut R) -> NodeFingers {

    let x_id = net.index_to_node(x_i).unwrap().clone();
    let target_ids_left = gen_left_target_ids(x_id, l);
    let target_ids_right = gen_right_target_ids(x_id, &net, l, &mut rng);

    let mut nf = NodeFingers::new(x_id, &target_ids_left, &target_ids_right);
    
    nf
}

pub fn init_fingers<R: Rng>(net: &Network<RingKey>, 
                l: usize, mut rng: &mut R) -> Vec<NodeFingers> {

    let mut res_fingers = Vec::new();
    for x_i in 0 .. net.igraph.node_count() {
        res_fingers.push(create_node_fingers(x_i, &net, l, &mut rng));
    }

    res_fingers
}


/// Make sure that a given chain is made of adjacent nodes.
fn verify_chain(chain: &NodeChain, net: &Network<RingKey>) -> bool {
    for i in 0 .. (chain.len() - 1) {
        let a = net.node_to_index(&chain[i]).unwrap();
        let b = net.node_to_index(&chain[i+1]).unwrap();
        if !net.igraph.contains_edge(a,b) {
            return false
        }
    }
    true
}

/*
fn verify_fingers(x_id: RingKey, chord_fingers: &ChordFingers, 
          net: &Network<RingKey>) -> bool {

    let l = chord_fingers.right_positive.len();

    // A function to get the top element:
    let check_chain = |chain: &NodeChain| (chain[chain.len() - 1] == x_id) &&
        verify_chain(chain, &net);

    let mut res: bool = true;

    res &= check_chain(&chord_fingers.left);

    for i in 0 .. l {
        res &= check_chain(&chord_fingers.right_positive[i]);
        res &= check_chain(&chord_fingers.right_negative[i]);
        res &= check_chain(&chord_fingers.right_randomized[i]);
        res &= check_chain(&chord_fingers.fully_randomized[i]);
    }

    for nei_connector in chord_fingers.neighbor_connectors.iter() {
        for chain in nei_connector {
            res &= check_chain(&chain);
        }
    }
    res

}
*/


/// Add an id to chain. Eliminate cycle if created.
fn add_id_to_chain(chain: &mut NodeChain, id: RingKey) {
    match chain.iter().position(|&x| x == id) {
        None => {
            chain.push(id);
        },
        Some(position) => {
            chain.resize(position + 1, 0);
        }
    };
}



/// Perform one iteration of fingers for all nodes
fn iter_fingers(net: &Network<RingKey>, 
                mut fingers: &mut Vec<NodeFingers>, l: usize) -> bool {

    // Check if any finger has changed:
    let mut has_changed = false;

    // Keep iterating until no changes happen:
    for x_i in 0 .. net.igraph.node_count() {
        let x_id = net.index_to_node(x_i).unwrap().clone();
        // Every node sends an UpdateRequest, and gets back an UpdateResponse message.

        for remote_schain in fingers[x_i].all_schains() {
            let remote_i = net.node_to_index(&remote_schain.final_id).unwrap();

            if x_i == remote_i {
                continue;
            }


            // Get two mutable indices (x_i and remote_i):
            let (m_x_i, m_remote_i) = match index_pair(&mut fingers, x_i, remote_i) {
                Pair::Two(m_x_i,m_remote_i) => (m_x_i, m_remote_i),
                _ => panic!("Invalid index pair: {}, {}", x_i, remote_i),
            };
            
            // UpdateRequest:
            // Every finger of x_id will get all of x_id's fingers.
            has_changed |= m_remote_i.update_by_fingers(&m_x_i, 
                       remote_schain.length, l);

            // UpdateResponse:
            // x_id will get all of the fingers of his fingers
            has_changed |= m_x_i.update_by_fingers(&m_remote_i,
                        remote_schain.length, l);
                
        }
    }

    has_changed
}


/// Get to converging state of fingers for all the network.
pub fn converge_fingers(net: &Network<RingKey>, 
             mut fingers: &mut Vec<NodeFingers>, l: usize) {

    // First iteration: We insert all edges:
    for x_i in 0 .. net.igraph.node_count() {
        for neighbor_i in net.igraph.neighbors(x_i) {
            let neighbor_id = net.index_to_node(neighbor_i).unwrap().clone();
            // Note that that information about the origin of the change
            // is contained as the last element of the NodeChain.

            let schain = SemiChain {
                final_id: neighbor_id,
                length: 1,
            };
            fingers[x_i].update(&schain,l);
        }
    }

    println!("Iter fingers...");
    while iter_fingers(&net, &mut fingers, l) {
        println!("Iter fingers...");
    }

}



fn create_semi_routes_node(x_i: usize, net: &Network<RingKey>, 
                           fingers: &Vec<NodeFingers>) -> SemiRoutesArray {

    let mut semi_routes_array = SemiRoutesArray::new();

    for schain in fingers[x_i].all_schains() {
        let mut semi_route: Vec<SemiChain> = Vec::new();
        semi_route.push(schain.clone());
        semi_routes_array.insert_sroute(semi_route.clone());

        // Concat with another iteration of SemiChain 
        // (Implementing Neighbor of Neighbor method):
        
        let target_i = net.node_to_index(&schain.final_id).unwrap();
        for next_schain in fingers[target_i].all_schains() {
            semi_route.push(next_schain);
            semi_routes_array.insert_sroute(semi_route.clone());
            semi_route.pop();
        }
    }
    semi_routes_array.index();
    semi_routes_array
}

pub fn create_semi_routes(net: &Network<RingKey>, 
                          fingers: &Vec<NodeFingers>) -> Vec<SemiRoutesArray> {

    let mut res_vec = Vec::new();
    for x_i in 0 .. net.igraph.node_count() {
        res_vec.push(create_semi_routes_node(x_i,&net, &fingers));
    }
    res_vec
}

/// Returns a length of a found path between src_id to dst_id, or 
/// None if no path was found.
pub fn find_path(src_id: RingKey, dst_id: RingKey, net: &Network<RingKey>, 
                 semi_routes: &Vec<SemiRoutesArray>) -> Option<usize> {

    // let mut semi_path: NodeChain = NodeChain::new();
    let mut cur_id = src_id;
    // semi_path.push(cur_id);
    let mut length: usize = 0;
    while cur_id != dst_id {
        let cur_semi_routes = &semi_routes[net.node_to_index(&cur_id).unwrap()];
        let semi_route = cur_semi_routes.find_closest_left(dst_id);
        let final_id = sroute_final_id(semi_route);
        if final_id == cur_id {
            return None;
        }

        for schain in semi_route {
            length += schain.length;
            // semi_path.push(schain.final_id);
        }

        cur_id = final_id;
    }
    Some(length)
}



/// Generate a random graph to be used with chord.
/// Graph nodes are of type RingKey.
pub fn random_net_chord<R: Rng>(num_nodes: usize, num_neighbors: usize, l: usize, rng: &mut R) 
        -> Network<RingKey> {

    // Maximum key in the ring:
    let max_key = 2_u64.pow(l as u32);

    // A hash set to make sure we don't have duplicate keys.
    let mut chosen_keys: HashSet<RingKey> = HashSet::new();

    // We can't have too many nodes with respect to the keyspace.
    // We stay below sqrt(keyspace_size), to avoid collisions.
    assert!(num_nodes < (max_key as f64).sqrt() as usize, "Too many nodes!");
    assert!(num_nodes > 0, "We should have at least one node!");

    let mut net = Network::<RingKey>::new();

    // Insert num_nodes nodes with random keys:
    for _ in 0 .. num_nodes {
        let rand_key: Range<RingKey> = Range::new(0,max_key);
        let mut node_key = rand_key.ind_sample(rng);
        while chosen_keys.contains(&node_key) {
            node_key = rand_key.ind_sample(rng);
        }
        chosen_keys.insert(node_key.clone());
        net.add_node(node_key);
    }

    // Add a straight line, to ensure connectivity.
    // Possibly change this later to a random tree.
    for v in 0 .. num_nodes - 1 {
        net.igraph.add_edge(v, v + 1, 1);
        // println!("add_edge {}, {}",v,v + 1);
    }

    // Connect node v to about num_neighbors previous nodes:
    // This should ensure connectivity, even if num_neighbors is small.
    for v in 0 .. num_nodes {
        for _ in 0 .. num_neighbors {
            let rand_node: Range<usize> = Range::new(0,num_nodes);
            let u = rand_node.ind_sample(rng);
            if u == v  {
                // Avoid self loops
                continue
            }
            if net.igraph.contains_edge(v,u) {
                // Already has this edge.
                continue
            }
            // Add edge:
            net.igraph.add_edge(v,u,1);
            // println!("add_edge {}, {}",v,u);
        }
    }
    net
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_d() {
        let l = 42;
        assert!(vdist(1u64,2,l) == 1);
        assert!(vdist(1u64,101,l) == 100);
        assert!(vdist(2_u64.pow(l as u32) - 1,1,l) == 2);
        assert!(vdist(2_u64.pow(l as u32) - 1,0,l) == 1);
        assert!(vdist(1,0,l) == 2_u64.pow(l as u32) - 1);
    }

    #[test]
    fn test_add_cyc() {
        // Check add:
        assert!(add_cyc(0,1,5) == 1);
        assert!(add_cyc(1,1,5) == 2);
        assert!(add_cyc(30,1,5) == 31);
        assert!(add_cyc(31,1,5) == 0);

        // Check sub:
        assert!(add_cyc(2,-1,5) == 1);
        assert!(add_cyc(1,-1,5) == 0);
        assert!(add_cyc(0,-1,5) == 31);
        assert!(add_cyc(31,-1,30) == 30);
    }



    #[test]
    fn test_inner_lexicographic() {
        // Make sure that vectors participate inside
        // lexicographic comparison.
        let d = (1,2,vec![3,6]);
        let a = (1,2,vec![3,4]);
        let b = (1,2,vec![3,5]);
        let c = (1,2,vec![3,6]);

        assert!(a < b);
        assert!(a < c);
        assert!(a < d);

        let aa = (1,2,vec![4,4]);
        assert!(aa > a);

        let m = (5,2,vec![1,1]);
        assert!(m > a);
    }


    #[test]
    fn test_add_id_to_chain_basic() {
        let mut chain = vec![1,2,3,4,5];
        add_id_to_chain(&mut chain, 3);
        assert!(chain == vec![1,2,3]);

        let mut chain = vec![1,2,3];
        add_id_to_chain(&mut chain, 3);
        assert!(chain == vec![1,2,3]);

        let mut chain = vec![1,2,3];
        add_id_to_chain(&mut chain, 4);
        assert!(chain == vec![1,2,3,4]);

        let mut chain = vec![1,2,3];
        add_id_to_chain(&mut chain, 1);
        assert!(chain == vec![1]);

        let mut chain = vec![1,2];
        add_id_to_chain(&mut chain, 1);
        assert!(chain == vec![1]);

        let mut chain = vec![1];
        add_id_to_chain(&mut chain, 1);
        assert!(chain == vec![1]);

    }

    #[test]
    fn test_chord_basic() {
        let seed: &[_] = &[1,2,3,4,9];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        let num_nodes = 5;
        let num_neighbors = 2;
        let l: usize = 6; // Size of keyspace
        let net = random_net_chord(num_nodes,num_neighbors,l,&mut rng);
        let mut fingers = init_fingers(&net,l, &mut rng);
        converge_fingers(&net, &mut fingers,l);
        let semi_routes = create_semi_routes(&net, &fingers);

        // let route_chains = get_route_chains(&net, &fingers, l);

        for index_a in 0 .. num_nodes {
            for index_b in index_a + 1 .. num_nodes {
                // Try to find a path:
                let src_id = net.index_to_node(index_a).unwrap().clone();
                let dst_id = net.index_to_node(index_b).unwrap().clone();
                let path_len = find_path(src_id, dst_id, &net, 
                                      &semi_routes).unwrap();

                /*
                // Make sure that all nodes in the path are connected by edges in the graph:
                for i in 0 .. (path.len() - 1) {
                    let a = net.node_to_index(&path[i]).unwrap();
                    let b = net.node_to_index(&path[i+1]).unwrap();
                    assert!(net.igraph.contains_edge(a,b));
                }
                */

                // Make sure that path begins with src_id and ends with dst_id:
                // assert!(path[0] == src_id);
                // assert!(path[path.len() - 1] == dst_id);
            }
        }
    }

}

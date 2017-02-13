extern crate rand;
extern crate ordered_float;

use network::{random_net, Network};
use coords::{build_coords, choose_landmarks, is_coord_unique};
use coord_mappers::{coord_to_ring_all_pairs, coord_to_ring, 
    approx_max_dist};
use statistic::spearman;
use random_util::choose_k_nums;

use self::rand::{Rng, StdRng};
use self::rand::distributions::{IndependentSample, Range};
use self::ordered_float::OrderedFloat;

// A trait alias for the distance function:
pub trait DistAFunc: Fn(usize,usize,&Vec<Vec<u64>>,&Vec<usize>) -> f64 {}
impl<T: Fn(usize,usize,&Vec<Vec<u64>>,&Vec<usize>) -> f64> DistAFunc for T {}

pub fn check_unique_coord(l: u32) {

    // Set up graph parameters:
    // let l: u32 = 16;
    let n: usize = ((2 as u64).pow(l)) as usize;
    let num_neighbours: usize = (1.5 * (n as f64).ln()) as usize;
    let num_landmarks: usize = (((l*l) as u32)/3) as usize;

    println!("n = {}",n);
    println!("num_neighbours = {}",num_neighbours);
    println!("num_landmarks = {}",num_landmarks);

    let seed: &[_] = &[1,2,3,4,5];
    let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
    println!("Creating the network...");
    let net = random_net(n,num_neighbours,&mut rng);
    let landmarks = choose_landmarks(&net,num_landmarks, &mut rng);
    println!("Iterating through coordinates");
    let coords = build_coords(&net, &landmarks);

    if coords.is_none() {
        println!("graph is not connected! Aborting.");
        return
    }

    let is_unique = is_coord_unique(&(coords.unwrap()));
    println!("is_unique = {}",is_unique);

}

pub fn check_ring_nums() {
    println!("{}",coord_to_ring(&vec![1,2,3]));
    println!("{}",coord_to_ring(&vec![2,3,1]));
    println!("{}",coord_to_ring(&vec![3,1,2]));

    println!("----------");

    println!("{}",coord_to_ring(&vec![5,2,4,8,5,9,4,1,1,5,8,7,3]));
    println!("{}",coord_to_ring(&vec![5,3,4,8,5,9,4,1,1,5,8,7,3]));
    println!("{}",coord_to_ring(&vec![5,4,4,8,5,9,4,1,1,5,8,7,3]));
    println!("{}",coord_to_ring(&vec![5,4,4,8,5,8,3,1,1,5,8,7,3]));
    println!("{}",coord_to_ring(&vec![5,4,4,8,5,8,3,1,1,5,8,55,3]));
    println!("{}",coord_to_ring(&vec![5,3,4,8,5,8,3,1,1,5,8,55,3]));
    println!("{}",coord_to_ring(&vec![5,3,4,8,5,3,3,1,1,5,8,55,3]));
    println!("{}",coord_to_ring(&vec![5,3,4,8,5,3,3,2,1,5,8,55,3]));
    println!("{}",coord_to_ring(&vec![5,4,4,8,6,2,3,2,1,5,7,55,2]));
}

/// Check the quality of distance approximation with the function fdisk using the pearson
/// monotonic correlation coefficient (Comparing to the real network distance)
pub fn check_approx_dist<F>(num_iters: u32, fdist: F, 
    net: &Network<usize>, coords: &Vec<Vec<u64>>, landmarks: &Vec<usize>, mut rng: &mut StdRng) 
    where F: DistAFunc {

    let mut dists = Vec::new();
    let mut adists = Vec::new();

    for _ in 0 .. num_iters {
        let node_pair: Vec<usize> = choose_k_nums(2,net.igraph.node_count(),&mut rng)
            .into_iter().collect::<Vec<_>>();

        let (u,v) = (node_pair[0], node_pair[1]);

        // Push real distance between u and v on the network:
        dists.push(net.dist(u,v).unwrap() as f64);

        // Push approximated distance between u and v:
        adists.push(fdist(u,v,&coords,&landmarks));
    }

    println!("spearman: {}",spearman(&dists,&adists).unwrap());
}

/// Try to find a path in the network between src_node and dst_node.
/// Returns None if path was not found, or Some(path_length)
fn try_route(src_node: usize, dst_node: usize, 
         amount_close: usize, net: &Network<usize>, 
         coords: &Vec<Vec<u64>>, landmarks: &Vec<usize>) -> Option<u64> {
    // Node distance function:
    let node_dist = |x,y| approx_max_dist(x,y,&coords, &landmarks);
    let mut num_hops = 0;

    let mut cur_node = src_node;

    // println!("------------------------");
    // println!("Routing from {} to {}",src_node, dst_node); 
    
    while cur_node != dst_node {
        let (new_cur_node, new_dist): (usize, u64) = 
            net.closest_nodes(cur_node)
            .take(amount_close)
            .min_by_key(|&(i, _)| OrderedFloat(node_dist(dst_node, i))).unwrap();

        if new_cur_node == cur_node {
            return None;
        }
        num_hops += new_dist;
        cur_node = new_cur_node;
    }
    Some(num_hops)
}


///
/// Check the success rate of routing in the network.
/// amount_close is the amount of close nodes every node keeps.
/// iters is the amount of iterations for this check.
pub fn check_routing(net: &Network<usize>, coords: &Vec<Vec<u64>>, landmarks: &Vec<usize>, 
         mut rng: &mut StdRng, amount_close: usize, iters: usize) {

    // Amount of routing failures:
    let mut num_route_fails: usize = 0;
    // Sum of path length (Used for average later)
    let mut sum_route_length: u64 = 0;

    for _ in 0 .. iters {
        let node_pair: Vec<usize> = choose_k_nums(2,net.igraph.node_count(),&mut rng)
            .into_iter().collect::<Vec<_>>();

        let num_hops = try_route(node_pair[0], node_pair[1],
                            amount_close, &net, &coords, &landmarks);

        match num_hops {
            Some(num) => sum_route_length += num,
            None => num_route_fails += 1,
        };
    }

    let num_route_success = iters - num_route_fails;
    let mean_route_length = (sum_route_length as f64) / (num_route_success as f64);

    let success_ratio = (num_route_success as f64) / (iters as f64);

    println!("success_ratio = {}", success_ratio);
    println!("mean_route_length = {}", mean_route_length);
}

/// Try to find a path in the network between src_node and dst_node.
/// Using a variation of random walk.
/// Returns None if path was not found, or Some(path_length)
fn try_route_random(src_node: usize, dst_node: usize, 
         amount_close: usize, net: &Network<usize>, 
         coords: &Vec<Vec<u64>>, landmarks: &Vec<usize>,
         mut rng: &mut StdRng) -> Option<u64> {

    // Node distance function:
    let node_dist = |x,y| approx_max_dist(x,y,&coords, &landmarks);
    let mut num_hops = 0;

    let mut cur_node = src_node;

    // println!("------------------------");
    // println!("Routing from {} to {}",src_node, dst_node); 
    
    while cur_node != dst_node {
        let closest_nodes: Vec<(usize, u64)> = net.closest_nodes(cur_node)
            .take(amount_close)
            .collect::<Vec<_>>();

        let &(mut new_cur_node, mut new_dist): &(usize, u64) = closest_nodes.iter()
            .min_by_key(|&&(i, dist)| OrderedFloat(node_dist(dst_node, i))).unwrap();

        while new_cur_node == cur_node {
            let rand_range: Range<usize> = Range::new(0,closest_nodes.len());
            let tup = closest_nodes[rand_range.ind_sample(rng)];
            new_cur_node = tup.0;
            new_dist = tup.1;
        }
        num_hops += new_dist;
        cur_node = new_cur_node;
    }
    Some(num_hops)
}

///
/// Check the success rate of routing in the network.
/// amount_close is the amount of close nodes every node keeps.
/// iters is the amount of iterations for this check.
pub fn check_routing_random(net: &Network<usize>, coords: &Vec<Vec<u64>>, landmarks: &Vec<usize>, 
         mut rng: &mut StdRng, amount_close: usize, iters: usize) {

    // Amount of routing failures:
    let mut num_route_fails: usize = 0;
    // Sum of path length (Used for average later)
    let mut sum_route_length: u64 = 0;

    for _ in 0 .. iters {
        let node_pair: Vec<usize> = choose_k_nums(2,net.igraph.node_count(),&mut rng)
            .into_iter().collect::<Vec<_>>();

        let num_hops = try_route_random(node_pair[0], node_pair[1],
                            amount_close, &net, &coords, &landmarks, &mut rng);

        match num_hops {
            Some(num) => sum_route_length += num,
            None => num_route_fails += 1,
        };
    }

    let num_route_success = iters - num_route_fails;
    let mean_route_length = (sum_route_length as f64) / (num_route_success as f64);

    let success_ratio = (num_route_success as f64) / (iters as f64);

    println!("success_ratio = {}", success_ratio);
    println!("mean_route_length = {}", mean_route_length);
}

/// Check if there are any local minima for network coordinates.
pub fn check_local_minima(net: &Network<usize>, coords: &Vec<Vec<u64>>, landmarks: &Vec<usize>, 
         mut rng: &mut StdRng, amount_close: usize, iters: usize) {

    // Node distance function:
    let node_dist = |x,y| approx_max_dist(x,y,&coords, &landmarks);

    let mut sum_ratio: f64 = 0.0;

    for _ in 0 .. iters {
        let rand_range: Range<usize> = Range::new(0,net.igraph.node_count());
        let dst_node = rand_range.ind_sample(rng);

        let mut num_not_minimum = 0;

        for src_node in 0 .. net.igraph.node_count() {
            if src_node == dst_node {
                continue
            }
            let found_better: bool = net.closest_nodes(src_node)
                .take(amount_close)
                .any(|(i, dist)| node_dist(i, dst_node) < node_dist(src_node, dst_node));

            if found_better {
                num_not_minimum += 1;
            }
        }

        let ratio_not_minimum = 
            (num_not_minimum as f64) / ((net.igraph.node_count() - 1) as f64);

        sum_ratio += ratio_not_minimum;
    }

    println!("success_ratio = {}", sum_ratio / (iters as f64));
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_basic() {
        let seed: &[_] = &[1,2,3,4,5];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        let net = random_net(60,5,&mut rng);
        let landmarks = choose_landmarks(&net,10,&mut rng);
        let coords = build_coords(&net, &landmarks);

        is_coord_unique(&(coords.unwrap()));
    }
}

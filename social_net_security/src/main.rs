extern crate rand;

use rand::{StdRng, Rng};
use rand::distributions::{Range, IndependentSample};

const NET_GOOD_RATIO: f64 = 0.9;
const LIST_RATIO_THRESHOLD: f64 = 0.8;

fn rand_is_good<R: Rng>(rng: &mut R) -> bool {
    let range_is_good: Range<f64> = Range::new(0.0,1.0);
    range_is_good.ind_sample(rng) <= NET_GOOD_RATIO
}

/// Perform one iteration for list.
fn iter_list<R: Rng>(list: &mut Vec<u8>, rng: &mut R) {

    let index_range: Range<usize> = Range::new(0, list.len());

    /*
    let init_index = index_range.ind_sample(rng);

    let is_good = if list[init_index] == 1 {
        // If we start with a bad node, we are expected to get a bad node.
        false
    } else {
        // Generate a random new node, good with probability NET_GOOD_RATIO.
        rand_is_good(rng)
    };
    */

    let is_good = rand_is_good(rng);

    // Assign result to a random index, discarding previous contents:
    list[index_range.ind_sample(rng)] = match is_good {
        true => 0,
        false => 1,
    }
}

fn calc_list_good_ratio(list: &[u8]) -> f64 {
    ((list.len() - list.iter().map(|&x| x as usize).sum::<usize>()) as f64) 
        / (list.len() as f64)
}

/// Run a full experiment with a new list.
/// l is the length of the list.
fn run_list<R: Rng>(l: usize, rng: &mut R) {
    let mut list = vec![0; l];
    while calc_list_good_ratio(&list) > LIST_RATIO_THRESHOLD {
        iter_list(&mut list, rng);
        // println!("list = {:?}", list);
    }
}

fn main() {
    let seed: &[_] = &[1,2,3,4];
    let mut rng: StdRng = rand::SeedableRng::from_seed(seed);

    println!("Before run_list...");
    run_list(400, &mut rng);
    println!("After run_list...");
}

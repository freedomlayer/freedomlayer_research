extern crate rand;

use rand::{StdRng, Rng};
use rand::distributions::{Range, IndependentSample};

const NET_GOOD_RATIO: f64 = 0.9;
const LIST_RATIO_THRESHOLD: f64 = 0.7;

/// Perform one iteration for list.
fn iter_list<R: Rng>(list: &mut Vec<u8>, rng: &mut R) {

    // Generate a random new node, good with probability NET_GOOD_RATIO.
    let range_is_good: Range<f64> = Range::new(0.0,1.0);
    let is_good: bool = range_is_good.ind_sample(rng) <= NET_GOOD_RATIO;

    let index_range: Range<usize> = Range::new(0, list.len());
    list[index_range.ind_sample(rng)] = match is_good {
        true => 0,
        false => 1,
    }
}

fn calc_list_good_ratio(list: &[u8]) -> f64 {
    ((list.len() - list.iter().sum::<u8>() as usize) as f64) 
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
    run_list(120, &mut rng);
    println!("After run_list...");
}

extern crate rand;
extern crate approximate_net;

use self::rand::{StdRng};
use approximate_net::{gen_elems, calc_mins, approx_size_harmonic};

fn main() {
    let seed: &[_] = &[1,2,3,4,5];
    let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
    let elems = gen_elems(200000, &mut rng);
    let mins = calc_mins(&elems, 50);
    let approx_size = approx_size_harmonic(&mins);
    println!("approx_size = {}", approx_size);
}

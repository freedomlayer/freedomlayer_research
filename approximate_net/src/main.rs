extern crate rand;
extern crate approximate_net;

use self::rand::{StdRng};
// use self::rand::distributions::{IndependentSample, Range};
use approximate_net::{gen_elems, calc_mins, approx_size_harmonic};

fn main() {
    let seed: &[_] = &[1,2,3,4,5,6];
    let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
    let num_mins = 40;
    let num_iters = 100;
    let num_elems = 10000;

    let mut total_serror = 0;

    for _ in 0 .. num_iters {
        // let num_elems_range = Range::<usize>::new(1000, 1000000);
        // let num_elems = num_elems_range.ind_sample(&mut rng);
        let elems = gen_elems(num_elems, &mut rng);
        let mins = calc_mins(&elems, num_mins);
        let approx_size = approx_size_harmonic(&mins);

        total_serror += (approx_size - elems.len()).pow(2);
    }

    let variance = total_serror / num_iters;

    println!("variance = {}", variance);
    println!("err ratio = {}", (variance as f64).sqrt() / num_elems as f64);


}

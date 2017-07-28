extern crate rand;
extern crate approximate_net;

use self::rand::{StdRng};
use approximate_net::{
    eval_approx_size_funcs};

use approximate_net::approx_funcs::approx_size_harmonic;

fn main() {
    let seed: &[_] = &[1,2,3,4,5,6];
    let mut rng: StdRng = rand::SeedableRng::from_seed(seed);

    println!("Calculating variance for approx_size_harmonic...");
    let num_iters = 100;
    let num_mins = 40;
    let num_elems = 100000;

    let err_ratios = eval_approx_size_funcs(num_iters, 
                                            num_mins, 
                                            num_elems, 
                                            &[&approx_size_harmonic], 
                                            &mut rng);
    println!("err_ratio = {}", err_ratios[0]);

}

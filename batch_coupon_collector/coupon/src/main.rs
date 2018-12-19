use rand::{Rng, thread_rng};


struct Coupons {
    coupons: Vec<bool>,
}

impl Coupons {
    fn new(n: usize) -> Self {
        Coupons {
            coupons: vec![false; n],
        }
    }

    fn is_done(&self) -> bool {
        for b in &self.coupons {
            if !b {
                return false;
            }
        }
        true
    }

    fn batch_coupons<R: Rng>(&mut self, k: usize, rng: &mut R) -> bool {
        for _ in 0 .. k {
            let i = rng.gen_range(0, self.coupons.len());
            self.coupons[i] = true;
        }
        self.is_done()
    }
}


fn main() {
    let mut rng = thread_rng();

    let n = 2 << 15;
    let mut coupons = Coupons::new(n);
    let mut num_iters = 0;
    while !coupons.batch_coupons(1, &mut rng) {
        num_iters += 1;
    }

    println!("num_iters = {}", num_iters);
    println!("nlogn = {}", (n as f64) * (n as f64).ln());
}





/// Calculate harmonic mean of given values
fn harmonic_mean(vals: &[f64]) -> f64 {
    let fsum: f64 = vals.iter()
        .map(|&x| 1.0 / x)
        .sum();

    (vals.len() as f64) / fsum
}

pub fn approx_size_harmonic(mins: &[u64]) -> usize {
    let trans = mins.iter()
        .map(|&m| (u64::max_value() / m) - 1)
        .map(|x| x as f64)
        .collect::<Vec<f64>>();

    harmonic_mean(&trans) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approx_size_harmonic() {
        let mins = &[111,222,333,4,555];
        approx_size_harmonic(mins);
    }

}

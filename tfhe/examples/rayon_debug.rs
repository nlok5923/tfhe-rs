use rayon::prelude::*;

pub fn main() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let mut values: Vec<u64> = (0..1000).map(|_| rng.gen::<u64>()).collect();

    values.par_iter_mut().for_each(|x| *x *= 2);

    println!("{values:?}")
}
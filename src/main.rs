use fast_poisson::{Poisson2D, PoissonVariable2D};

fn main() {
    let dim = 2.0;
    let r = 1.0;
    let k = 30;
    let seed = 123123;
    let points = Poisson2D::new()
        .with_dimensions([dim, dim], r)
        .with_seed(seed)
        .with_samples(k)
        .generate();
    println!("num of points {:?}", points.len());

    let points = PoissonVariable2D::new()
        .with_dimensions([dim, dim], r)
        .with_seed(seed)
        .with_samples(k)
        .generate();
    println!("num of points {:?}", points.len());
}

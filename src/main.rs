use fast_poisson::{Poisson2D, PoissonVariable2D};

fn main() {
    let points = Poisson2D::new()
        .with_dimensions([500.0, 500.0], 32.0)
        .with_samples(30)
        .generate();
    println!("num of points {:?}", points.len());

    let points = PoissonVariable2D::new()
        .with_dimensions([10.0, 10.0], 2.0)
        .with_seed(123123)
        .with_samples(30)
        .generate();
    println!("num of points {:?}", points.len());
}

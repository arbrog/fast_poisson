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
    println!("num of points {:?}", points);

    // too close check
    for point0 in points.iter() {
        for point1 in points.iter() {
            if point0 != point1 {
                let dist_squared = point0
                    .iter()
                    .zip(point1.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>();
                if dist_squared < r {
                    println!(
                        "{:?} and {:?} are too close. r^2 = {dist_squared}",
                        point0, point1
                    );
                }
            }
        }
    }
}

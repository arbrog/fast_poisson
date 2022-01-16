use bracket_noise::prelude::*;
use fast_poisson::PoissonVariable2D;

fn main() {
    let dim = 3_f64;
    let r_min = 1.0;
    let r_max = 2.0;
    let k = 30;
    let seed = 123123;

    let mut noise = FastNoise::seeded(seed);
    noise.set_noise_type(NoiseType::PerlinFractal);
    noise.set_fractal_type(FractalType::FBM);
    noise.set_fractal_octaves(5);
    noise.set_fractal_gain(0.6);
    noise.set_fractal_lacunarity(2.0);
    noise.set_frequency(2.0);

    let min_cell_size: f64 = r_min / 2_f64.sqrt();

    let grid_size: usize = [dim, dim]
        .iter()
        .map(|n| (n / min_cell_size).ceil() as usize)
        .product();

    let mut radius_map = vec![0.0; grid_size];
    for (i, cell) in radius_map.iter_mut().enumerate() {
        let y = i / grid_size;
        let x = i % grid_size;
        let value: f64 = noise
            .get_noise(
                x as f32 / (dim as f32) * 2_f32,
                y as f32 / (dim as f32) * 2_f32,
            )
            .into();
        *cell = (value * (r_max - r_min)) + r_min;
    }
    // println!("{:#?}", radius_map);

    let points = PoissonVariable2D::new()
        .with_dimensions([dim, dim], (r_min, r_max))
        .with_seed(seed)
        .with_samples(k)
        .with_noise(radius_map)
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

                let scalar: f64 = noise
                    .get_noise(
                        point0[0] as f32 / dim as f32 * 2_f32,
                        point0[1] as f32 / dim as f32 * 2_f32,
                    )
                    .into();
                let radius = ((scalar * (r_max - r_min)) + r_min).powi(2);
                if dist_squared < radius {
                    println!(
                        "{:?} and {:?} are too close. d^2 = {dist_squared} while r^2 is {radius}",
                        point0, point1
                    );
                }
            }
        }
    }
}

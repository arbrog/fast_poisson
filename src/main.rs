use bracket_noise::prelude::*;
use fast_poisson::PoissonVariable2D;
use image::Rgb;
extern crate image;

fn main() {
    let dim = 512.0;
    let r_min = 1.0;
    let r_max = 8.0;
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

    let noise_grid_width = (dim / min_cell_size).ceil() as usize;

    let grid_size: usize = [dim, dim]
        .iter()
        .map(|n| (n / min_cell_size).ceil() as usize)
        .product();

    let mut radius_map = vec![0.0; grid_size];
    for (i, cell) in radius_map.iter_mut().enumerate() {
        let y = i / grid_size;
        let x = i % grid_size;
        let value: f64 = (noise.get_noise(
            x as f32 / (noise_grid_width as f32) * 2_f32,
            y as f32 / (noise_grid_width as f32) * 2_f32,
        ) + 0.5_f32)
            .into();
        *cell = (value * (r_max - r_min)) + r_min;
    }

    let mut raw_noise_buffer =
        image::ImageBuffer::new(noise_grid_width as u32, noise_grid_width as u32);

    for (x, y, pixel) in raw_noise_buffer.enumerate_pixels_mut() {
        let value: f64 = (noise.get_noise(
            x as f32 / (noise_grid_width as f32) * 2_f32,
            y as f32 / (noise_grid_width as f32) * 2_f32,
        ) + 0.5_f32)
            .into();
        let value: u8 = 255 - ((value * 255_f64) as u8);
        *pixel = image::Rgb([value, value, value]);
    }
    // println!("{:#?}", radius_map);

    let points = PoissonVariable2D::new()
        .with_dimensions([dim, dim], (r_min, r_max))
        .with_seed(seed)
        .with_samples(k)
        .with_noise(radius_map)
        .generate();
    println!("num of points {:?}", points.len());

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut points_buffer = image::ImageBuffer::new(dim as u32, dim as u32);

    // Iterate over the coordinates and pixels of the image
    for (_, _, pixel) in points_buffer.enumerate_pixels_mut() {
        *pixel = image::Rgb([255_u8, 255_u8, 255_u8]);
    }

    for point in points.iter() {
        points_buffer.put_pixel(
            point[0].floor() as u32,
            point[1].floor() as u32,
            Rgb([0_u8, 0, 0]),
        );
    }

    raw_noise_buffer.save("noise.png").unwrap();
    points_buffer.save("points.png").unwrap();
}

// Copyright 2021 Travis Veazey
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// https://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Generate a Poisson disk distribution.
//!
//! This is an implementation of Bridson's ["Fast Poisson Disk Sampling"][Bridson] algorithm in
//! arbitrary dimensions.
//!
//!  * Iterator-based generation lets you leverage the full power of Rust's
//!    [Iterators](Iterator)
//!  * Lazy evaluation of the distribution means that even complex Iterator chains are as fast as
//!    O(N); with other libraries operations like mapping into another struct become O(N²) or more!
//!  * Using Rust's const generics allows you to consume the distribution with no additional
//!    dependencies
//!
//! # Features
//!
//! These are the optional features you can enable in your Cargo.toml:
//!
//!  * `single_precision` changes the output, and all of the internal calculations, from using
//!    double-precision `f64` to single-precision `f32`. Distributions generated with the
//!    `single-precision` feature are *not* required nor expected to match those generated without
//!    it.
//!  * `small_rng` changes the internal PRNG used to generate the distribution: By default
//!    [`Xoshiro256StarStar`](rand_xoshiro::Xoshiro256StarStar) is used, but with this feature
//!    enabled then [`Xoshiro128StarStar`](rand_xoshiro::Xoshiro128StarStar) is used instead. This
//!    reduces the memory used for the PRNG's state from 256 bits to 128 bits, and may be more
//!    performant for 32-bit systems.
//!
//! # Requirements
//!
//! This library requires Rust 1.51.0 or later, as it relies on [const generics] to return
//! fixed-length points (e.g. [x, y] or [x, y, z]) without adding additional external dependencies
//! to your code.
//!
//! # Examples
//!
//! ```
//! use fast_poisson::Poisson2D;
//!
//! // Easily generate a simple `Vec`
//! # // Some of these examples look a little hairy because we have to accomodate for the feature
//! # // `single_precision` in doctests, which changes the type of the returned values.
//! # #[cfg(not(feature = "single_precision"))]
//! let points: Vec<[f64; 2]> = Poisson2D::new().generate();
//! # #[cfg(feature = "single_precision")]
//! # let points: Vec<[f32; 2]> = Poisson2D::new().generate();
//!
//! // To fill a box, specify the width and height:
//! let points = Poisson2D::new().with_dimensions([100.0, 100.0], 5.0);
//!
//! // Leverage `Iterator::map` to quickly and easily convert into a custom type in O(N) time!
//! # #[cfg(not(feature = "single_precision"))]
//! struct Point {
//!     x: f64,
//!     y: f64,
//! }
//! # #[cfg(feature = "single_precision")]
//! # struct Point { x: f32, y: f32 }
//! let points = Poisson2D::new().iter().map(|[x, y]| Point { x, y });
//!
//! // With the `From` trait implemented for `Point`, we can directly convert into `Vec<Point>`
//! # #[cfg(not(feature = "single_precision"))]
//! impl From<[f64; 2]> for Point {
//!     fn from(point: [f64; 2]) -> Point {
//!         Point {
//!             x: point[0],
//!             y: point[1],
//!         }
//!     }
//! }
//! # #[cfg(feature = "single_precision")]
//! # impl From<[f32; 2]> for Point {
//! #     fn from(point: [f32; 2]) -> Point {
//! #         Point {
//! #             x: point[0],
//! #             y: point[1],
//! #         }
//! #     }
//! # }
//! let points: Vec<Point> = Vec::from(Poisson2D::new());
//! let points: Vec<Point> = Poisson2D::new().into();
//!
//! // Distributions are lazily evaluated; here only 5 points will be calculated!
//! let points = Poisson2D::new().iter().take(5);
//!
//! // `Poisson` can be directly consumed in for loops:
//! for point in Poisson2D::new() {
//!     println!("X: {}; Y: {}", point[0], point[1]);
//! }
//! ```
//!
//! Higher-order Poisson disk distributions are generated just as easily:
//! ```
//! use fast_poisson::{Poisson, Poisson3D, Poisson4D};
//!
//! // 3-dimensional distribution
//! let points_3d = Poisson3D::new().iter();
//!
//! // 4-dimensional distribution
//! let mut points_4d = Poisson4D::new();
//! // To achieve desired levels of performance, you should set a larger radius for higher-order
//! // distributions
//! points_4d.with_dimensions([1.0; 4], 0.2);
//! let points_4d = points_4d.iter();
//!
//! // For more than 4 dimensions, use `Poisson` directly:
//! let mut points_7d = Poisson::<7>::new();
//! points_7d.with_dimensions([1.0; 7], 0.6);
//! let points_7d = points_7d.iter();
//! ```
//!
//! # Upgrading
//!
//! ## 0.4.x
//!
//! This version is 100% backwards-compatible with 0.3.x and 0.2.0, however `fast_poisson` has been
//! relicensed as of this version.
//!
//! Several bugs were identified and fixed in the underlying algorithms; as a result, distributions
//! generated with 0.4.0 will *not* match those generated in earlier versions.
//!
//! ## 0.3.x
//!
//! This version adds no breaking changes and is backwards-compatible with 0.2.0.
//!
//! ## 0.2.0
//!
//! This version adds some breaking changes:
//!
//! ### 2 dimensions no longer assumed
//!
//! In version 0.1.0 you could directly instantiate `Poisson` and get a 2-dimensional distribution.
//! Now you must specifiy that you want 2 dimensions using either `Poisson<2>` or [`Poisson2D`].
//!
//! ### Returned points are arrays
//!
//! In version 0.1.0 the distribution was returned as an iterator over `(f64, f64)` tuples
//! representing each point. To leverage Rust's new const generics feature and support arbitrary
//! dimensions, the N-dimensional points are now `[f64; N]` arrays.
//!
//! ### Builder pattern
//!
//! Use the build pattern to instantiate new distributions. This will not work:
//! ```compile_fail
//! # use fast_poisson::Poisson2D;
//! let poisson = Poisson2D {
//!     width: 100.0,
//!     height: 100.0,
//!     radius: 5.0,
//!     ..Default::default()
//! };
//! let points = poisson.iter();
//! ```
//! Instead, leverage the new builder methods:
//! ```
//! # use fast_poisson::Poisson2D;
//! let mut poisson = Poisson2D::new();
//! poisson.with_dimensions([100.0; 2], 5.0);
//! let points = poisson.iter();
//! ```
//! This change frees me to make additional changes to how internal state is stored without necessarily
//! requiring additional changes to the API.
//!
//! [Bridson]: https://www.cct.lsu.edu/~fharhad/ganbatte/siggraph2007/CD2/content/sketches/0250.pdf
//! [Tulleken]: http://devmag.org.za/2009/05/03/poisson-disk-sampling/
//! [const generics]: https://blog.rust-lang.org/2021/03/25/Rust-1.51.0.html#const-generics-mvp
//! [small_rng]: https://docs.rs/rand/0.8.3/rand/rngs/struct.SmallRng.html

#[cfg(test)]
mod tests;

use rand::prelude::*;
use rand_distr::StandardNormal;
use std::iter::FusedIterator;

/// [`Poisson`] disk distribution in 2 dimensions
pub type Poisson2D = Poisson<2>;
/// [`Poisson`] disk distribution in 3 dimensions
pub type Poisson3D = Poisson<3>;
/// [`Poisson`] disk distribution in 4 dimensions
pub type Poisson4D = Poisson<4>;

#[cfg(not(feature = "single_precision"))]
type Float = f64;
#[cfg(feature = "single_precision")]
type Float = f32;

/// Poisson disk distribution in N dimensions
///
/// Distributions can be generated for any non-negative number of dimensions, although performance
/// depends upon the volume of the space: for higher-order dimensions you may need to [increase the
/// radius](Poisson::with_dimensions) to achieve the desired level of performance.
#[derive(Debug, Clone)]
pub struct Poisson<const N: usize> {
    /// Dimensions of the box
    dimensions: [Float; N],
    /// Radius around each point that must remain empty
    radius: Float,
    /// Seed to use for the internal RNG
    seed: Option<u64>,
    /// Number of samples to generate and test around each point
    num_samples: u32,
}

impl<const N: usize> Poisson<N> {
    /// Create a new Poisson disk distribution
    ///
    /// By default, `Poisson` will sample each dimension from the semi-open range [0.0, 1.0), using
    /// a radius of 0.1 around each point, and up to 30 random samples around each; the resulting
    /// output will be non-deterministic, meaning it will be different each time.
    ///
    /// See [`Poisson::with_dimensions`] to change the range and radius, [`Poisson::with_samples`]
    /// to change the number of random samples for each point, and [`Poisson::with_seed`] to produce
    /// repeatable results.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Specify the space to be filled and the radius around each point
    ///
    /// To generate a 2-dimensional distribution in a 5×5 square, with no points closer than 1:
    /// ```
    /// # use fast_poisson::Poisson2D;
    /// let mut points = Poisson2D::new().with_dimensions([5.0, 5.0], 1.0).iter();
    ///
    /// assert!(points.all(|p| p[0] >= 0.0 && p[0] < 5.0 && p[1] >= 0.0 && p[1] < 5.0));
    /// ```
    ///
    /// To generate a 3-dimensional distribution in a 3×3×5 prism, with no points closer than 0.75:
    /// ```
    /// # use fast_poisson::Poisson3D;
    /// let mut points = Poisson3D::new().with_dimensions([3.0, 3.0, 5.0], 0.75).iter();
    ///
    /// assert!(points.all(|p| {
    ///     p[0] >= 0.0 && p[0] < 3.0
    ///     && p[1] >= 0.0 && p[1] < 3.0
    ///     && p[2] >= 0.0 && p[2] < 5.0
    /// }));
    /// ```
    pub fn with_dimensions(&mut self, dimensions: [Float; N], radius: Float) -> &mut Self {
        self.dimensions = dimensions;
        self.radius = radius;

        self
    }

    /// Specify the PRNG seed for this distribution
    ///
    /// If no seed is specified then the internal PRNG will be seeded from entropy, providing
    /// non-deterministic and non-repeatable results.
    ///
    /// ```
    /// # use fast_poisson::Poisson2D;
    /// let points = Poisson2D::new().with_seed(0xBADBEEF).iter();
    /// ```
    pub fn with_seed(&mut self, seed: u64) -> &Self {
        self.seed = Some(seed);

        self
    }

    /// Specify the maximum samples to generate around each point
    ///
    /// Note that this is not specifying the number of samples in the resulting distribution, but
    /// rather sets the maximum number of attempts to find a new, valid point around an existing
    /// point for each iteration of the algorithm.
    ///
    /// A higher number may result in better space filling, but may also slow down generation.
    ///
    /// ```
    /// # use fast_poisson::Poisson3D;
    /// let points = Poisson3D::new().with_samples(40).iter();
    /// ```
    pub fn with_samples(&mut self, samples: u32) -> &Self {
        self.num_samples = samples;

        self
    }

    /// Returns an iterator over the points in this distribution
    ///
    /// ```
    /// # use fast_poisson::Poisson3D;
    /// let points = Poisson3D::new();
    ///
    /// for point in points.iter() {
    ///     println!("{:?}", point);
    /// }
    /// ```
    #[must_use]
    pub fn iter(&self) -> PoissonIter<N> {
        PoissonIter::new(self.clone())
    }

    /// Generate the points in this Poisson distribution, collected into a [`Vec`](std::vec::Vec).
    ///
    /// Note that this method does *not* consume the `Poisson`, so you can call it multiple times
    /// to generate multiple `Vec`s; if you have specified a seed, each one will be identical,
    /// whereas they will each be unique if you have not (see [`Poisson::with_seed`]).
    ///
    /// ```
    /// # use fast_poisson::Poisson2D;
    /// let mut poisson = Poisson2D::new();
    ///
    /// let points1 = poisson.generate();
    /// let points2 = poisson.generate();
    ///
    /// // These are not identical because no seed was specified
    /// assert!(points1.iter().zip(points2.iter()).any(|(a, b)| a != b));
    ///
    /// poisson.with_seed(1337);
    ///
    /// let points3 = poisson.generate();
    /// let points4 = poisson.generate();
    ///
    /// // These are identical because a seed was specified
    /// assert!(points3.iter().zip(points4.iter()).all(|(a, b)| a == b));
    /// ```
    pub fn generate(&self) -> Vec<Point<N>> {
        self.iter().collect()
    }
}

impl<const N: usize> Default for Poisson<N> {
    fn default() -> Self {
        Poisson::<N> {
            dimensions: [1.0; N],
            radius: 0.1,
            seed: None,
            num_samples: 30,
        }
    }
}

impl<const N: usize> IntoIterator for Poisson<N> {
    type Item = Point<N>;
    type IntoIter = PoissonIter<N>;

    fn into_iter(self) -> Self::IntoIter {
        PoissonIter::new(self)
    }
}

impl<const N: usize> IntoIterator for &Poisson<N> {
    type Item = Point<N>;
    type IntoIter = PoissonIter<N>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// For convenience allow converting to a Vec directly from Poisson
impl<T, const N: usize> From<Poisson<N>> for Vec<T>
where
    T: From<[Float; N]>,
{
    fn from(poisson: Poisson<N>) -> Vec<T> {
        poisson.iter().map(|point| point.into()).collect()
    }
}

/// A Point is simply an array of Float values
type Point<const N: usize> = [Float; N];

/// A Cell is the grid coordinates containing a given point
type Cell<const N: usize> = [isize; N];

#[cfg(not(feature = "small_rng"))]
type Rand = rand_xoshiro::Xoshiro256StarStar;
#[cfg(feature = "small_rng")]
type Rand = rand_xoshiro::Xoshiro128StarStar;

/// An iterator over the points in the Poisson disk distribution
pub struct PoissonIter<const N: usize> {
    /// The distribution from which this iterator was built
    distribution: Poisson<N>,
    /// The RNG
    rng: Rand,
    /// The size of each cell in the grid
    cell_size: Float,
    /// The grid stores spatially-oriented samples for fast checking of neighboring sample points
    grid: Vec<Option<Point<N>>>,
    /// A list of valid points that we have not yet visited
    active: Vec<Point<N>>,
}

impl<const N: usize> PoissonIter<N> {
    /// Create an iterator over the specified distribution
    fn new(distribution: Poisson<N>) -> Self {
        // We maintain a grid of our samples for faster radius checking
        let cell_size = distribution.radius / (N as Float).sqrt();

        // If we were not given a seed, generate one non-deterministically
        let mut rng = match distribution.seed {
            None => Rand::from_entropy(),
            Some(seed) => Rand::seed_from_u64(seed),
        };

        // Calculate the amount of storage we'll need for our n-dimensional grid, which is stored
        // as a single-dimensional array.
        let grid_size: usize = distribution
            .dimensions
            .iter()
            .map(|n| (n / cell_size).ceil() as usize)
            .product();

        // We have to generate an initial point, just to ensure we've got *something* in the active list
        let mut first_point = [0.0; N];
        for (i, dim) in first_point.iter_mut().zip(distribution.dimensions.iter()) {
            *i = rng.gen::<Float>() * dim;
        }

        let mut iter = PoissonIter {
            distribution,
            rng,
            cell_size,
            grid: vec![None; grid_size],
            active: Vec::new(),
        };
        // Don't forget to add our initial point
        iter.add_point(first_point);

        iter
    }

    /// Add a point to our pattern
    fn add_point(&mut self, point: Point<N>) {
        // Add it to the active list
        self.active.push(point);

        // Now stash this point in our grid
        let idx = self.point_to_idx(point);
        self.grid[idx] = Some(point);
    }

    /// Convert a point into grid cell coordinates
    fn point_to_cell(&self, point: Point<N>) -> Cell<N> {
        let mut cell = [0_isize; N];

        for i in 0..N {
            cell[i] = (point[i] / self.cell_size) as isize;
        }

        cell
    }

    /// Convert a cell into a grid vector index
    fn cell_to_idx(&self, cell: Cell<N>) -> usize {
        cell.iter()
            .zip(self.distribution.dimensions.iter())
            .fold(0, |acc, (pn, dn)| {
                acc * (dn / self.cell_size) as usize + *pn as usize
            })
    }

    /// Convert a point into a grid vector index
    fn point_to_idx(&self, point: Point<N>) -> usize {
        self.cell_to_idx(self.point_to_cell(point))
    }

    /// Generate a random point between `radius` and `2 * radius` away from the given point
    fn generate_random_point(&mut self, around: Point<N>) -> Point<N> {
        // Pick a random distance away from our point
        let dist = self.distribution.radius * (1.0 + self.rng.gen::<Float>());

        // Generate a randomly distributed vector
        let mut vector: [Float; N] = [0.0; N];
        for i in vector.iter_mut() {
            *i = self.rng.sample(StandardNormal);
        }
        // Now find this new vector's magnitude
        let mag = vector.iter().map(|&x| x.powi(2)).sum::<Float>().sqrt();

        // Dividing each of the vector's components by `mag` will produce a unit vector; then by
        // multiplying each component by `dist`, we'll have a vector pointing `dist` away from the
        // origin. If we then add each of those components to our point, we'll have effectively
        // translated our point by `dist` in a randomly chosen direction.
        // Conveniently, we can do all of this in just one step!
        let mut point = [0.0; N];
        let translate = dist / mag; // compute this just once!
        for i in 0..N {
            point[i] = around[i] + vector[i] * translate;
        }

        point
    }

    /// Returns true if the point is within the bounds of our space.
    ///
    /// This is true if 0 ≤ point[i] < dimensions[i]
    fn in_space(&self, point: Point<N>) -> bool {
        point
            .iter()
            .zip(self.distribution.dimensions.iter())
            .all(|(p, d)| *p >= 0. && p < d)
    }

    /// Returns true if the cell is within the bounds of our grid.
    ///
    /// This is true if 0 ≤ `cell[i]` ≤ `ceiling(space[i] / cell_size)`
    fn in_grid(&self, cell: Cell<N>) -> bool {
        cell.iter()
            .zip(self.distribution.dimensions.iter())
            .all(|(c, d)| *c >= 0 && *c < (*d / self.cell_size).ceil() as isize)
    }

    /// Returns true if there is at least one other sample point within `radius` of this point
    fn in_neighborhood(&self, point: Point<N>) -> bool {
        let cell = self.point_to_cell(point);

        // We'll compare to distance squared, so we can skip the square root operation for better performance
        let r_squared = self.distribution.radius.powi(2);

        for mut carry in 0.. {
            let mut neighbor = cell;

            // We can add our current iteration count to visit each neighbor cell
            for i in (&mut neighbor).iter_mut() {
                // We clamp our addition to the range [-2, 2] for each cell
                *i += carry % 5 - 2;
                // Since we modulo by 5 to get the right range, integer division by 5 "advances" us
                carry /= 5;
            }

            if carry > 0 {
                // If we've "overflowed" then we've already tested every neighbor cell
                return false;
            }
            if !self.in_grid(neighbor) {
                // Skip anything beyond the bounds of our grid
                continue;
            }

            if let Some(point2) = self.grid[self.cell_to_idx(neighbor)] {
                let neighbor_dist_squared = point
                    .iter()
                    .zip(point2.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<Float>();

                if neighbor_dist_squared < r_squared {
                    return true;
                }
            }
        }

        // Rust can't tell the previous loop will always reach one of the `return` statements...
        false
    }
}

impl<const N: usize> Iterator for PoissonIter<N> {
    type Item = Point<N>;

    fn next(&mut self) -> Option<Point<N>> {
        while !self.active.is_empty() {
            let i = self.rng.gen_range(0..self.active.len());

            for _ in 0..self.distribution.num_samples {
                // Generate up to `num_samples` random points between radius and 2*radius from the current point
                let point = self.generate_random_point(self.active[i]);

                // Ensure we've picked a point inside the bounds of our rectangle, and more than `radius`
                // distance from any other sampled point
                if self.in_space(point) && !self.in_neighborhood(point) {
                    // We've got a good one!
                    self.add_point(point);

                    return Some(point);
                }
            }

            self.active.swap_remove(i);
        }

        None
    }
}

impl<const N: usize> FusedIterator for PoissonIter<N> {}

// Hacky way to include README in doc-tests, but works until #[doc(include...)] is stabilized
// https://github.com/rust-lang/cargo/issues/383#issuecomment-720873790
#[cfg(doctest)]
mod test_readme {
    macro_rules! external_doc_test {
        ($x:expr) => {
            #[doc = $x]
            extern "C" {}
        };
    }

    external_doc_test!(include_str!("../README.md"));
}

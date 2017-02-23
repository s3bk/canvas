extern crate canvas;
extern crate image;
extern crate nalgebra;
extern crate num;

use canvas::plot::{Figure, XY, Parametric};
use nalgebra::Point2;
use num::complex::Complex64;

#[test]
fn test_plot_luma_sin() {
    Figure::new(-1.0 .. 1.0, -1.1 .. 1.1)
        .sample(XY::new(Box::new(|x| (1.0/x).sin()), -1.0 .. 1.0), 100_000)
        .draw::<image::GrayImage>(400, 200)
        .save("data/test_plot_luma_sin.png")
        .unwrap();
}

#[test]
fn test_plot_rgb_sin() {
    Figure::new(-1.0 .. 1.0, -1.1 .. 1.1)
        .sample(XY::new(Box::new(|x| (1.0/x).sin()), -1.0 .. 1.0), 100_000)
        .draw::<image::RgbImage>(400, 200)
        .save("data/test_plot_rgb_sin.png")
        .unwrap();
}


#[test]
fn test_plot_rgb_nautilus() {
    use std::f64::consts::PI;
    
    fn nautilus(rot: f64) -> Box<Fn(f64) -> Point2<f64>> {
        let k: f64 = 2. * ((5_f64.sqrt() + 1.) / 2.).ln() / PI;
        let s = Complex64::new(0.0, rot).exp();
        Box::new(move |c| {
            let z = Complex64::new(0.5, 0.5) + 0.5 * s * (c * Complex64::new(k, 1.)).exp();
            Point2::new(z.re, z.im)
        })
    }
    
    Figure::new(-5.0 .. 5.0, -5.0 .. 5.0)
        .sample(Parametric::new(nautilus(0.0 * PI), -10.0 .. 10.0), 1_00_000)
        .sample(Parametric::new(nautilus(0.5 * PI), -10.0 .. 10.0), 1_00_000)
        .sample(Parametric::new(nautilus(1.0 * PI), -10.0 .. 10.0), 1_00_000)
        .sample(Parametric::new(nautilus(1.5 * PI), -10.0 .. 10.0), 1_00_000)
        .draw::<image::RgbImage>(500, 500)
        .save("data/test_plot_rgb_nautilus.png")
        .unwrap();
}

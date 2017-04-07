extern crate canvas;
extern crate image;
extern crate num;
extern crate tuple;

use canvas::plot::{Figure, XY, Parametric};
use canvas::array::{Array, RowMajor};
use canvas::colormap::grayscale;
use num::complex::Complex64;
use tuple::*;

#[test]
fn test_plot_luma_sin() {
    let canvas = Figure::new(-1.0 .. 1.0, -1.1 .. 1.1)
        .sample(XY::new(Box::new(|x: f32| (1.0/x).sin()), -1.0 .. 1.0), 100_000)
        .draw::<Array<Vec<f32>, RowMajor>>(400, 200);
        
    grayscale(&canvas).save("data/test_plot_luma_sin.png").unwrap();
}

#[test]
fn test_plot_rgb_sin() {
    let canvas = Figure::new(-1.0 .. 1.0, -1.1 .. 1.1)
        .sample(XY::new(Box::new(|x: f32| (1.0/x).sin()), -1.0 .. 1.0), 100_000)
        .draw::<Array<Vec<f32>, RowMajor>>(400, 200);
        
    grayscale(&canvas).save("data/test_plot_rgb_sin.png").unwrap();
}


#[test]
fn test_plot_rgb_nautilus() {
    use std::f64::consts::PI;
    
    fn nautilus(rot: f64) -> Box<Fn(f64) -> T2<f64, f64>> {
        let k: f64 = 2. * ((5_f64.sqrt() + 1.) / 2.).ln() / PI;
        let s = Complex64::new(0.0, rot).exp();
        Box::new(move |c| {
            let z = Complex64::new(0.5, 0.5) + 0.5 * s * (c * Complex64::new(k, 1.)).exp();
            T2(z.re, z.im)
        })
    }
    
    let canvas = Figure::new(-5.0 .. 5.0, -5.0 .. 5.0)
        .sample(Parametric::new(nautilus(0.0 * PI), -10.0 .. 10.0), 1_00_000)
        .sample(Parametric::new(nautilus(0.5 * PI), -10.0 .. 10.0), 1_00_000)
        .sample(Parametric::new(nautilus(1.0 * PI), -10.0 .. 10.0), 1_00_000)
        .sample(Parametric::new(nautilus(1.5 * PI), -10.0 .. 10.0), 1_00_000)
        .draw::<Array<Vec<f32>, RowMajor>>(500, 500);
        
    grayscale(&canvas).save("data/test_plot_rgb_nautilus.png").unwrap();
}

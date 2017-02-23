#![feature(non_ascii_idents)]
#![feature(conservative_impl_trait)]

extern crate canvas;
extern crate image;
extern crate nalgebra;

use nalgebra::Vector2;
use canvas::plot::{Figure, XY, Parametric};
use canvas::integrate::IntegratedFunction;
use image::RgbaImage;


fn duffing(ɛ: f64, λ: f64, Ω: f64, α: f64, β: f64) -> impl Fn(f64, Vector2<f64>) -> Vector2<f64> {
    move |t, s| {
        Vector2::new(
            s.y,
            ɛ * (Ω + t).cos() - λ * s.y - α * s.x - β * s.x.powi(3)
        )
    }
}

#[test]
fn test_duffing() {
    Figure::new(-4.0 .. 4.0, -6.0 .. 6.0)
    .trace(
        IntegratedFunction::new(
            duffing(7.72, 0.2, 1.0, 0.0, 1.0),
            Vector2::new(1.0, 1.0),
            1e-3
        ),
        1000_000
    )
    .draw::<image::RgbImage>(1600, 800)
    .save("data/test_duffing.png")
    .unwrap();
}

#![feature(inclusive_range_syntax)]

extern crate canvas;
extern crate tuple;
extern crate simd;
extern crate math;
extern crate fmath;

use canvas::plot::{Figure};
use canvas::array::{Array, RowMajor};
use tuple::T2;
use simd::x86::avx::f32x8;
use math::real::Real;

fn sin_cos(x: f32x8) -> (f32x8, f32x8) {
    let y = x * x;
    (
        fmath::poly_f32x8_avx(fmath::consts::POLY32_SIN_8_PI, y) * x,
        fmath::poly_f32x8_avx(fmath::consts::POLY32_COS_8_PI, y)
    )
}

fn main() {
    let mut canvas: Figure<f32, Array<Vec<f32>, RowMajor>> = Figure::new(-f32::PI .. f32::PI, -f32::PI .. f32::PI, (1024, 1024));

    // d/dx cos(x) * sin(y) = -sin(x) * sin(y)
    // d/dy cos(x) * sin(y) = cos(x) * cos(y)
    
    for i in -9 ... 9i8 {
        let c = f32x8::splat(i as f32 * 0.1);
        eprint!("{} ", i);
        canvas.contour_gradient(
            move |T2(x, y): T2<f32x8, f32x8>| {
                let (x_sin, x_cos) = sin_cos(x);
                let (y_sin, y_cos) = sin_cos(y);

                (x_cos * y_sin - c, T2(-x_sin * y_sin, x_cos * y_cos))
            },
            100_000,
            10       
        );
    }
    eprintln!(" saving");
    canvas.grayscale(Some(200.)).save("data/contour_gradient.png").unwrap();
}

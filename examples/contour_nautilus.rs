#![feature(inclusive_range_syntax)]

extern crate canvas;
extern crate tuple;
extern crate simd;
extern crate math;
extern crate fmath;

use canvas::plot::{Figure};
use canvas::array::{Array, RowMajor};
use tuple::*;
use simd::x86::avx::f32x8;
use math::real::Real;

fn sin_cos(x: f32x8) -> (f32x8, f32x8) {
    let y = x * x;
    (
        fmath::poly_f32x8_avx(fmath::consts::POLY32_SIN_8_PI, y) * x,
        fmath::poly_f32x8_avx(fmath::consts::POLY32_COS_8_PI, y)
    )
}

fn mod1(x: f32) -> f32 {
    if x > 0. {
        x.fract()
    } else {
        1.0 + x.fract()
    }
}

fn main() {
    let mut canvas: Figure<f32, Array<Vec<f32>, RowMajor>> = Figure::new(-10.0 .. 10.0, -10.0 .. 10.0, (1024, 1024));

    let Phi = (5.0f32.sqrt() + 1.0) / 2.0;
    let i_s = (0.5 * f32::PI) / Phi.ln();
    for s in (0.0, 0.5).into_elements() {
        canvas.contour_gradient(
            move |T2(x, y): T2<f32, f32>| {
                let r = (x*x + y*y).sqrt() + 0.0001;
                let n = r.ln();
                let phi = y.atan2(x) / (2.0 * f32::PI);

                let f = mod1(n - phi + 0.01 + s) - 0.01;

                let e = T2(x, y) / r;
                
                (f, e / r)
            },
            100_000,
            20
        );
    }
    
    canvas.grayscale(Some(100.)).save("data/contour_nautilus.png").unwrap();
}

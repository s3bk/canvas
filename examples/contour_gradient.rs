#![feature(inclusive_range_syntax)]

extern crate canvas;
extern crate tuple;
extern crate simd;
extern crate math;

use canvas::plot::{Figure};
use canvas::array::{Array, RowMajor};
use tuple::T2;
use simd::x86::avx::f32x8;
use math::real::Real;
use math::builder::Builder;
use math::avx::avx_jit;
use math::diff::diff;
use std::env;

fn main() {
    let expr = env::args().skip(1).next().unwrap_or_else(|| "cos(x) * sin(y) - c".into());
    
    let mut canvas: Figure<f32, Array<Vec<f32>, RowMajor>> = Figure::new(-4. .. 4., -4. .. 4., (512, 512));

    // d/dx cos(x) * sin(y) = -sin(x) * sin(y)
    // d/dy cos(x) * sin(y) = cos(x) * cos(y)

    let b = Builder::new();
    let f = b.parse(&expr).expect("failed to parse");
    let df_dx = diff(&b, &f, "x").unwrap();
    let df_dy = diff(&b, &f, "y").unwrap();

    let J2 = b.add(b.mul(df_dx.clone(), df_dx.clone()), b.mul(df_dy.clone(), df_dy.clone()));
    let J_inv_x = b.div(df_dx, J2.clone()).unwrap();
    let J_inv_y = b.div(df_dy, J2).unwrap();
    let code = avx_jit((&f, &J_inv_x, &J_inv_y), ["x", "y", "c"]);
    
    for i in -9 ... 9i8 {
        let c = f32x8::splat(i as f32);
        eprint!("{} ", i);
        canvas.contour_gradient(
            |T2(x, y): T2<f32x8, f32x8>| {
                let (p, px, py) = code.call(&[x, y, c]);
                (p, T2(px, py))
            },
            10_000,
            10
        );
    }
    eprintln!(" saving");
    canvas.grayscale(Some(200.)).save("data/contour_gradient.png").unwrap();
}

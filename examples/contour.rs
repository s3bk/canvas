#![feature(inclusive_range_syntax)]

extern crate canvas;
extern crate tuple;
extern crate math;
extern crate simd;

use simd::x86::avx::f32x8;
use canvas::plot::{Figure};
use canvas::array::{Array, RowMajor};
use tuple::*;
use math::builder::Builder;
use math::avx::avx_jit;
use std::env;


fn main() {
    let expr = env::args().skip(1).next().unwrap_or_else(|| "cos(x) * sin(y) - c / 10".into());
    let mut canvas: Figure<f32, Array<Vec<f32>, RowMajor>> = Figure::new(-4.0 .. 4.0, -4.0 .. 4.0, (512, 512));

    let b = Builder::new();
    let f = b.parse(&expr).expect("failed to parse");

    let code = avx_jit((&f, ), ["x", "y", "c"]);

    // d/dx cos(x) * sin(y) = -sin(x) * sin(y)
    // d/dy cos(x) * sin(y) = cos(x) * cos(y)
    
    for i in -9 ... 9i8 {
        let c = f32x8::splat(i as f32);
        canvas.contour(|T2(x, y): T2<f32, f32>| {
            let vars = [f32x8::splat(x), f32x8::splat(y), c];
            let T8(y, ..) = code.call(&vars).0.into();
            y
        });
    }
        
    canvas.grayscale(None).save("data/contour.png").unwrap();
}

#![feature(inclusive_range_syntax)]

extern crate canvas;
extern crate image;
extern crate tuple;

use canvas::plot::{Figure};
use canvas::array::{Array, RowMajor};
use tuple::*;

fn main() {
    let mut canvas: Figure<f32, Array<Vec<f32>, RowMajor>> = Figure::new(-1.0 .. 4.0, -1.0 .. 4.0, (512, 512));
    // f(x, y) = cos(x+y) + sin(x+y) * ((x-2)^2 + (y-1)^2)^-1
    
    for i in 0 .. 8 * (2+4) {
        let c = i as f32 * 0.125 - 2.0;
        canvas.contour(move |T2(x, y): T2<f32, f32>| {
            let (sin, cos) = (x+y).sin_cos();
            cos + sin / ((x-2.0).powi(2) + (y-1.0).powi(2)) - c
        });
    }
        
    canvas.grayscale().save("data/contour2.png").unwrap();
}

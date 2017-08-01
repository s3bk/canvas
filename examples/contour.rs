#![feature(inclusive_range_syntax)]

extern crate canvas;
extern crate image;
extern crate tuple;

use canvas::plot::{Figure};
use canvas::array::{Array, RowMajor};
use tuple::*;

fn main() {
    let mut canvas: Figure<f32, Array<Vec<f32>, RowMajor>> = Figure::new(-4.0 .. 4.0, -4.0 .. 4.0, (512, 512));

    // d/dx cos(x) * sin(y) = -sin(x) * sin(y)
    // d/dy cos(x) * sin(y) = cos(x) * cos(y)
    
    for i in -9 ... 9i8 {
        let c = i as f32 * 0.1;
        canvas.contour(move |T2(x, y): T2<f32, f32>| x.cos() * y.sin() - c);
    }
        
    canvas.grayscale().save("data/contour.png").unwrap();
}

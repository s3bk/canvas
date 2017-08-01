#![feature(inclusive_range_syntax)]

extern crate canvas;
extern crate image;
extern crate tuple;

use canvas::plot::{Figure};
use canvas::array::{Array, RowMajor};
use tuple::*;

fn main() {
    let mut canvas: Figure<f32, Array<Vec<f32>, RowMajor>> = Figure::new(-1.0 .. 4.0, -1.0 .. 4.0, (512, 512));
    // g(x, y) = (x-2)^2 + (y-1)^2
    // f(x, y) = cos(x+y) + sin(x+y) / g(x, y)
    //                  a                      b
    // df/dx = -2 (x-2) sin(x+y) / g(x, y)^2 - sin(x+y) + cos(x+y) / g(x, y)
    // df/dy = -2 (y-1) sin(x+y) / g(x, y)^2 - sin(x+y) + cos(x+y) / g(x, y)

    for i in 0i8 .. 8 * (2+4) {
        let c = i as f32 * 0.125 - 2.0;
        eprint!("{:.3}\r", c);
        canvas.contour_gradient(
            move |T2(x, y): T2<f32, f32>| {
                let (sin, cos) = (x+y).sin_cos();
                let frac = 1.0 / ((x-2.0).powi(2) + (y-1.0).powi(2));
                let a = -2.0 * sin * frac.powi(2);
                let b = cos * frac - sin;
                (
                    cos + sin * frac - c,
                    T2(
                        (x - 2.0) * a + b,
                        (y - 1.0) * a + b
                    )
                )
            },
            10_000,
            100
        );
    }
    eprintln!(" done");
        
    canvas.grayscale(None).save("data/contour_gradient2.png").unwrap();
}

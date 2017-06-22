#![feature(non_ascii_idents)]
#![feature(conservative_impl_trait)]
#![feature(core_intrinsics)]
#![feature(box_syntax)]

extern crate canvas;
extern crate tuple;
extern crate num;
extern crate math;
extern crate fmath;
extern crate show;
extern crate image;

use canvas::plot::{LineStyle};
use tuple::T2;
use num::Num;
use canvas::plot::{Figure};
use canvas::array::{Array, RowMajor};
use canvas::canvas::{Canvas, Data, Meta};
use canvas::colormap;
use math::integrate::Integration;
use math::real::Real;
use fmath::*;
use image::RgbaImage;
use show::{Visible, Rotation};

fn cos(x: f32) -> f32 {
    let x = f32x8::splat(x);
    poly8_f32x8_avx(POLY32_COS_8_PI, x * x).extract(0)
}

#[allow(non_snake_case)]
#[inline]
fn duffing(ɛ: f32, λ: f32, _Ω: f32, α: f32, β: f32)
 -> impl Fn(f32, T2<f32, f32>) -> T2<f32, f32>
{
    use std::intrinsics::{fmul_fast};
    move |t, s| {
        unsafe {
            T2(
                s.1,
                fmul_fast(ɛ, cos(t))
                - fmul_fast(λ, s.1)
                - fmul_fast(s.0, α + fmul_fast(fmul_fast(s.0, s.0), β))
            )
        }
    }
}

pub struct Viewer {
    fig:    Figure<f32>,
    img:    RgbaImage,
    map:    Array<Vec<f32>, RowMajor>
}
impl Visible for Viewer {
    fn update(&mut self, t: f64) -> &RgbaImage {
        self.fig.draw_on(&mut self.map);
        colormap::map_to(&self.map, &*colormap::MAP_STEEL, &mut self.img);
        &self.img
    }
}

fn main() {
    let width = 1024;
    let height = 1024;

    let mut f = Figure::new(-4.0f32 .. 4.0, -6.0 .. 6.0);
    f.trace(
        Integration::new(
            duffing(7.72, 0.2, 1.0, 0.0, 1.0),
            T2(1.0, 1.0),
            0.0,
            1e-3
        ),
        100_000
    );
    
    let mut v = Viewer {
        fig:    f,
        img:    RgbaImage::new(width as u32, height as u32),
        map:    Array::new(RowMajor::new(width, height), vec![0.0f32; width * height])
    };
    v.show(Rotation::R0);
    
    v.img.save("duffing.png").unwrap();
}

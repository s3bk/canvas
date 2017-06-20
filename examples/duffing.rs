#![feature(non_ascii_idents)]
#![feature(conservative_impl_trait)]
#![feature(core_intrinsics)]
#![feature(box_syntax)]

extern crate canvas;
extern crate tuple;
extern crate num;
extern crate math;
extern crate fmath;

use canvas::plot::{TracedItem, LineStyle};
use tuple::T2;
use num::Num;
use canvas::plot::{Figure};
use canvas::array::{Array, RowMajor};
use canvas::canvas::{Canvas, Data, Meta};
use canvas::colormap;
use math::integrate::Integration;
use math::real::Real;
use fmath::*;

/// F: closure that defines the function to integrate
/// N: data type to work wit (typically f32 or f64)
pub struct IntegratedFunction<F, N: Num=f64> {
    func:   F,
    init:   T2<N, N>,
    step:   N,
}
impl<F, N: Num> IntegratedFunction<F, N> {
    pub fn new(f: F, init: T2<N, N>, step: N) -> IntegratedFunction<F, N> {
        IntegratedFunction {
            func:   f,
            init:   init,
            step:   step
        }
    }
}
impl<F, N: Real + Num + 'static> TracedItem<N> for IntegratedFunction<F, N>
    where F: Fn(N, T2<N, N>) -> T2<N, N>
{
    fn trace<'a>(&'a self) -> Box<Iterator<Item=T2<N, N>> + 'a> {
        box Integration::new(&self.func, self.init, N::zero(), self.step)
    }
}

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

fn main() {
    let width = 1024;
    let height = 1024;

    let mut buf = vec![0.0f32; width * height];
    
    let mut map = Array::new(RowMajor::new(width, height), buf);
    
    Figure::new(-4.0f32 .. 4.0, -6.0 .. 6.0)
    //.trace(
    .trace(
        IntegratedFunction::new(
            duffing(7.72, 0.2, 1.0, 0.0, 1.0),
            T2(1.0, 1.0),
            1e-3
        ),
        10_000_000
    )
    .draw_on(&mut map);
    
    map.run_mut(|meta, data| {
        data.map(|x| x.sqrt())
    });
    
    //colormap::map(&map, &&colormap::MAP_COLORFUL[..] as &colormap::ColorMap)
    colormap::grayscale(&map)
    .save("data/test_duffing.png")
    .unwrap();
}

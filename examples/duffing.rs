#![feature(non_ascii_idents)]
#![feature(conservative_impl_trait)]
#![feature(core_intrinsics)]
#![feature(box_syntax)]

extern crate canvas;
extern crate tuple;
extern crate num;
extern crate math;

use canvas::plot::TracedItem;
use tuple::T2;
use num::Num;
use canvas::plot::{Figure};
use canvas::array::{Array, RowMajor};
use canvas::canvas::{Canvas, Data, Meta, default};
use canvas::colormap;
use math::integrate::Integration;
use math::real::Real;

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


#[allow(non_snake_case)]
#[inline]
fn duffing(ɛ: f32, λ: f32, Ω: f32, α: f32, β: f32)
 -> impl Fn(f32, T2<f32, f32>) -> T2<f32, f32>
{
    use std::intrinsics::{fmul_fast, cosf32};
    move |t, s| {
        unsafe {
            T2(
                s.1,
                fmul_fast(ɛ, cosf32(t))
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
    
    Figure::new(-4.0 .. 4.0, -6.0 .. 6.0)
    .trace(
        IntegratedFunction::new(
            duffing(7.72, 0.2, 1.0, 0.0, 1.0),
            T2(1.0, 1.0),
            1e-3
        ),
        1_000_000,
        20.
    )
    .draw_on(&mut map);
    
    colormap::map(&map, &&colormap::MAP_COLORFUL[..] as &colormap::ColorMap)
    .save("data/test_duffing.png")
    .unwrap();
}

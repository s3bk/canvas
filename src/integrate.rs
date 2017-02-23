use std::ops::{Mul, Add};
use plot::TracedItem;
use nalgebra::{Vector2};

pub struct Integration<N, F> {
    f:  F,
    t:  f64,
    y:  N,
    h:  f64
}
impl<N, F> Integration<N, F> {
    pub fn new(t0: f64, dt: f64, s0: N, f: F) -> Integration<N, F> {
        Integration {
            f:  f,
            t:  t0,
            y:  s0,
            h:  dt
        }
    }
}
impl<N, F> Iterator for Integration<N, F> where
    N: Mul<f64, Output=N> + Add<Output=N> + Copy,
    F: Fn(f64, N) -> N {
    type Item = N;
    fn next(&mut self) -> Option<N> {
        let ref f = self.f;
        let t = self.t;
        let h = self.h;
        let h_half = h / 2.0;
        let h_third = h / 3.0;
        let h_sixth = h / 6.0;
    
        let y = self.y;
        let k1 = f(t, y);
        let k2 = f(t + h_half, y + k1 * h_half);
        let k3 = f(t + h_half, y + k2 * h_half);
        let k4 = f(t + h, y + k3 * h);
        
        self.y = y + (k1 + k4) * h_sixth + (k2 + k3) * h_third;
        self.t += h;
        
        Some(self.y)
    }
}

pub struct IntegratedFunction<F> {
    func:   F,
    init:   Vector2<f64>,
    step:   f64,
}
impl<F> IntegratedFunction<F> {
    pub fn new(f: F, init: Vector2<f64>, step: f64) -> IntegratedFunction<F> {
        IntegratedFunction {
            func:   f,
            init:   init,
            step:   step
        }
    }
}
impl<F> TracedItem for IntegratedFunction<F>
    where F: Fn(f64, Vector2<f64>) -> Vector2<f64>
{
    fn trace<'a>(&'a self) -> Box<Iterator<Item=Vector2<f64>> + 'a> {
        box Integration::new(0.0, self.step, self.init, &self.func)
    }
}

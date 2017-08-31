use rand::XorShiftRng;
use math::real::Real;

pub trait VRng<R> {
    fn next(&mut self) -> R;
}

pub struct DefaultRng {
    r: XorShiftRng
}
impl DefaultRng {
    pub fn new() -> DefaultRng {
        DefaultRng { r: XorShiftRng::new_unseeded() }
    }
}
impl<R: Real> VRng<R> for DefaultRng {
    fn next(&mut self) -> R {
        R::uniform01(&mut self.r)
    }
}

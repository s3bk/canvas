use tuple::{T2, T4, TupleElements};
use math::real::{Real};
use math::cast::Cast;
use std::fmt::Debug;
use pen::Pen;

#[derive(Copy, Clone, Debug)]
struct Cell<N: Debug> {
    x: usize, // x-coordinate
    y: usize, // y-coordinate
    d: usize, // cell width/height
    f_tl: N, // f(x,   y  )
    f_tr: N, // f(x+d, y  )
    f_bl: N, // f(x,   y+d)
    f_br: N  // f(x+d, y+d)
}
impl<N: Debug> Cell<N> {
    fn split<F>(self, f: F) -> T4<Cell<N>, Cell<N>, Cell<N>, Cell<N>>
        where F: Fn(usize, usize) -> N, N: Copy
    {
        let Cell { x, y, d, f_tl, f_tr, f_bl, f_br } = self;
        let d2 = d / 2;
        let f_t = f(x+d2, y+d);
        let f_b = f(x+d2, y);
        let f_l = f(x,    y+d2);
        let f_r = f(x+d,  y+d2);
        let f_m = f(x+d2, y+d2);

        //  y + d   f_tl   f_t   f_tr
        //              A      B
        //  y + d2  f_l    f_m   f_r
        //              C      D
        //  y       f_bl   f_b   f_br
        //
        //          x      x+d2  x+d
        T4(
            Cell { x: x,    y: y+d2, d: d2, f_tl: f_tl, f_tr:  f_t,  f_bl: f_l,  f_br: f_m  }, // top-left (A)
            Cell { x: x+d2, y: y+d2, d: d2, f_tl: f_t,  f_tr:  f_tr, f_bl: f_m,  f_br: f_r  }, // top-right (B)
            Cell { x: x,    y: y,    d: d2, f_tl: f_l,  f_tr:  f_m,  f_bl: f_bl, f_br: f_b  }, // bottom-right (C)
            Cell { x: x+d2, y: y,    d: d2, f_tl: f_m,  f_tr:  f_r,  f_bl: f_b,  f_br: f_br }  // bottom-right (D)
        )
    }
}
pub struct ContourPlot<F, D> {
    pub search_depth: u8,
    pub plot_depth: u8,
    pub pen: Pen<D>,
    pub func: F
}
impl<F, D, N> ContourPlot<F, D> where F: Fn(T2<usize, usize>) -> N, D: FnMut(T2<isize, isize>, f32), N: Real<Bool=bool> + Copy + Cast<f32>,
usize: Cast<N>
{
    pub fn run(&mut self) {
        let q = {
            let d = 1 << self.plot_depth;
            let f = |x, y| (self.func)(T2(x, y));
            Cell { x: 0, y: 0, d: d, f_tl: f(0, d), f_tr: f(d, d), f_bl: f(0, 0), f_br: f(d, 0) }
        };
        self.create_tree(q, 0);
    }
    fn create_tree(&mut self, q: Cell<N>, depth: u8) {
        if depth < self.search_depth {
            self.subdivide(q, depth);
        } else if self.contour_present(q) {
            if depth < self.plot_depth {
                self.subdivide(q, depth);
            } else {
                self.plot(q);
            }
        }
    }
    fn subdivide(&mut self, q: Cell<N>, depth: u8) {
        for q in q.split(|x, y| (self.func)(T2(x, y))).into_elements() {
            self.create_tree(q, depth+1);
        }
    }
    fn contour_present(&self, q: Cell<N>) -> bool {
        let zero = T4::splat(N::int(0));
        let values = T4(q.f_tl, q.f_tr, q.f_bl, q.f_br);
        // cheat a bit to compute this a lot more efficient
        let s = values.lt(zero).map(|b| b as u8);
        // all false (4*0) or true (4*1) means no contour,
        // so 1, 2 and 3 have to return true, while 0 and 4 return true.
        s.into_elements().sum::<u8>() & 3 != 0
    }
    fn plot(&mut self, q: Cell<N>) {
        let zero = |a: N, b: N| {
            let x = a / (a - b);
            if x.ge(N::int(0)) && x.le(N::int(1)) {
                Some(x)
            } else {
                None
            }
        };

        let x0 = q.x.cast().unwrap();
        let y0 = q.y.cast().unwrap();
        let d = q.d.cast().unwrap();

        let top    = zero(q.f_tl, q.f_tr).map(|a| T2(x0 + a * d, y0 + d    ));
        let bottom = zero(q.f_bl, q.f_br).map(|a| T2(x0 + a * d, y0        ));
        let left   = zero(q.f_bl, q.f_tl).map(|a| T2(x0,         y0 + a * d));
        let right  = zero(q.f_br, q.f_tr).map(|a| T2(x0 + d,     y0 + a * d));

        let mut iter = T4(top, bottom, left, right).into_elements().filter_map(|x| x).map(|x| x.cast().unwrap().into());
        match (iter.next(), iter.next()) {
            (Some(p0), Some(p1)) => self.pen.line(p0, p1),
            _ => ()
        }
    }
}

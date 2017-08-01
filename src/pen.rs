use std::mem::swap;
use tuple::T2;
use math::cast::Cast;

type N = f32;

pub struct Pen<F> {
    draw:   F,
    p:      T2<N, N>
}

#[inline(always)]
fn ipart(x: N) -> N {
    x.floor()
}

#[inline(always)]
fn round(x: N) -> N {
    x.round()
}

#[inline(always)]
fn fpart(x: N) -> N {
    x.fract()
}

#[inline(always)]
fn rfpart(x: N) -> N {
    1.0 - x.fract()
}

impl<F> Pen<F> where F: FnMut(T2<isize, isize>, N)
{
    pub fn new(draw: F) -> Pen<F> {
        Pen {
            draw:   draw,
            p:      T2(0.0, 0.0)
        }
    }
    pub fn line(&mut self, p0: T2<N, N>, p1: T2<N, N>) {
        self.move_to(p0);
        self.line_to(p1);
    }
    #[inline]
    pub fn move_to(&mut self, p: T2<N, N>) {
        self.p = p;
    }
    #[inline]
    pub fn line_to(&mut self, p: T2<N, N>) {
        let T2(mut x0, mut y0) = self.p;
        let T2(mut x1, mut y1) = p;
        let half = 0.5;
        let threshold = 0.1;
        
        if (x1 - x0).abs().max((y1 - y0).abs()) < threshold {
            return;
        }
        
        // save values
        self.p = p;
        
        // shortcut to draw pixels
        let mut draw = |x, y, v| {
            (self.draw)(T2(x, y), v);
        };
        
        // http://en.wikipedia.org/wiki/Xiaolin_Wu%27s_line_algorithm
        
        let steep = (y1 - y0).abs() > (x1 - x0).abs();
        
        if steep {
            swap(&mut x0, &mut y0);
            swap(&mut x1, &mut y1);
        }
        
        if x0 > x1 {
            swap(&mut x0, &mut x1);
            swap(&mut y0, &mut y1);
        }
        
        let dx = x1 - x0;
        let dy = y1 - y0;
        let gradient = if dx > 0. {
            dy / dx
        } else {
            1.0
        };
        
        // handle first endpoint
        let xend = round(x0);
        let yend = y0 + gradient * (xend - x0);
        let xgap = rfpart(x0 + half);
        let xpxl1: isize = xend.cast().unwrap();   //this will be used in the main loop
        let ypxl1: isize = ipart(yend).cast().unwrap();
        
        let a = xgap * fpart(yend);
        let b = xgap * rfpart(yend);
        
        if steep {
            draw(ypxl1,   xpxl1,  b);
            draw(ypxl1+1, xpxl1,  a);
        } else {
            draw(xpxl1, ypxl1,    b);
            draw(xpxl1, ypxl1+1,  a);
        }
        
        let mut intery = yend + gradient; // first y-intersection for the main loop
    
        // handle second endpoint
    
        let xend = round(x1);
        let yend = y1 + gradient * (xend - x1);
        let xgap = fpart(x1 + half);
        let xpxl2: isize = xend.cast().unwrap(); //this will be used in the main loop
        let ypxl2: isize = ipart(yend).cast().unwrap();
        
        let a = xgap * fpart(yend);
        let b = xgap * rfpart(yend);
        
        if steep {
            draw(ypxl2,   xpxl2,  b);
            draw(ypxl2+1, xpxl2,  a);
        } else {
            draw(xpxl2, ypxl2,   b);
            draw(xpxl2, ypxl2+1, a);
        }
        
        // main loop
    
        if steep {
            for x in xpxl1 + 1 .. xpxl2 {
                let a = fpart(intery);
                let py: isize = ipart(intery).cast().unwrap();
                
                draw(py,   x, 1.0 - a);
                draw(py+1, x, a);
                intery += gradient;
            }
        } else {
            for x in xpxl1 + 1 .. xpxl2 {
                let a = fpart(intery);
                let py: isize = ipart(intery).cast().unwrap();
                
                draw(x, py,   1.0 - a);
                draw(x, py+1, a);
                intery += gradient;
            }
        }
    }
}

/*

impl<Draw> Pen<Draw> where Draw: FnMut(Vector2<i32>, u32)
{
    pub fn new(draw: Draw) -> Pen<Draw> {
        Pen {
            draw:   draw,
            p:      Vector2::zero()
        }
    }
    #[inline]
    pub fn move_to(&mut self, p: T2<f32, f32>) {
        self.p = p;
    }
    #[inline]
    pub fn line_to(&mut self, p: T2<f32, f32>, intensity: u32) -> Line {
        let mut x0 = self.p.x;
        let mut y0 = self.p.y;
        let mut x1 = p.x;
        let mut y1 = p.y;
        
        if (x1 - x0).abs().max((y1 - y0).abs()) < 0.1 {
            return;
        }
        
        // save values
        self.p = p;
        
        // shortcut to draw pixels
        let mut draw = |x, y, v| {
            (self.draw)(Vector2::new(x, y), v);
        };
        
        // http://en.wikipedia.org/wiki/Xiaolin_Wu%27s_line_algorithm
        
        let steep = (y1 - y0).abs() > (x1 - x0).abs();
        
        if steep {
            swap(&mut x0, &mut y0);
            swap(&mut x1, &mut y1);
        }
        
        if x0 > x1 {
            swap(&mut x0, &mut x1);
            swap(&mut y0, &mut y1);
        }
        
        let dx = x1 - x0;
        let dy = y1 - y0;
        let gradient = if dx > 0.0 {
            dy / dx
        } else {
            1.0
        };
        
        let xend = round(x1);
        let yend = y1 + gradient * (xend - x1);
        let xpxl2 = xend as isize; //this will be used in the main loop
        let ypxl2 = ipart(yend) as isize;
        
        Line {
            start:      Vector2::new(x0, y0),
            end:        Vector2::new(x1, y1),
            gradient:   gradient,
            state:      LineState::Start,
            steep:      steep,
            intensity:  intensity
        }
    }
}

pub struct Line {
    start:      T2<f32, f32>,
    end:        T2<f32, f32>,
    gradient:   f32
    state:      LineState,
    steep:      bool,
    intensity:  u32,
    xpxl2:      u32
}

enum LineState {
    Start,
    Mid(usize, f32),
    End
}

impl Iterator for Line {
    type Item = [(usize, usize, f32); 2];
    
    fn next(&mut self) {
        match self.state {
            LineState::Start => {
                let intensity_f = intensity as f32;
        
                let xgap = rfpart(x0 + 0.5);
                let a = (xgap * fpart(yend) * intensity_f) as u32;
                let b = (xgap * rfpart(yend) * intensity_f) as u32;
                
                
                let intery = yend + gradient; // first y-intersection for the main loop
                
                self.state = LineState::Mid(xpxl1+1, intery);
                
                if steep {
                    Some([
                        (ypxl1,   xpxl1,  b),
                        (ypxl1+1, xpxl1,  a)
                    ])
                } else {
                    Some([
                        (xpxl1, ypxl1,    b),
                        (xpxl1, ypxl1+1,  a)
                    ])
                }
            },
            LineState::Mid(xpxl1, intery) => {
                if xpxl1 < xpxl2 {
                    self.state = LineState::Mid(xpxl1+1, intery+gradient);
                    let a = (fpart(intery) * intensity_f) as u32;
                    let py = ipart(intery) as isize;
                    if self.steep {
                        Some([
                            (py,   x, intensity - a),
                            (py+1, x, a)
                        ])
                    } else {
                        Some([
                            (x, py,   intensity - a),
                            (x, py+1, a)
                        ])
                    }
                } else {
                    self.state = LineState::End;
                    
                    let xgap = fpart(x1 + 0.5);
                    
                    let a = (xgap * fpart(yend) * intensity_f) as u32;
                    let b = (xgap * rfpart(yend) * intensity_f) as u32;
                    
                    if steep {
                        Some([
                            (ypxl2,   xpxl2,  b),
                            (ypxl2+1, xpxl2,  a)
                        ])
                    } else {
                        Some([
                            (xpxl2, ypxl2,   b),
                            (xpxl2, ypxl2+1, a)
                        ])
                    }
                }
            },
            LineState::End => None
        }

        // handle second endpoint
    
        
        // main loop
    
    }
}
*/

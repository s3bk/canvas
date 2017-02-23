use std::mem::swap;
use nalgebra::{Vector2};
use num::Zero;

pub struct Pen<Draw> {
    draw:   Draw,
    p:      Vector2<f64>
}

fn ipart(x: f64) -> f64 {
    x.floor()
}

fn round(x: f64) -> f64 {
    x.round()
}

fn fpart(x: f64) -> f64 {
    x.fract()
}

fn rfpart(x: f64) -> f64 {
    1.0 - x.fract()
}

impl<Draw> Pen<Draw> where Draw: FnMut(Vector2<i32>, f64)
{
    pub fn new(draw: Draw) -> Pen<Draw> {
        Pen {
            draw:   draw,
            p:      Vector2::zero()
        }
    }
    pub fn move_to(&mut self, p: Vector2<f64>) {
        self.p = p;
    }
    pub fn line_to(&mut self, p: Vector2<f64>) {
        let Vector2{x: mut x0, y: mut y0} = self.p;
        let Vector2{x: mut x1, y: mut y1} = p;
        
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
        
        // handle first endpoint
        let xend = round(x0);
        let yend = y0 + gradient * (xend - x0);
        let xgap = rfpart(x0 + 0.5);
        let xpxl1 = xend as i32;   //this will be used in the main loop
        let ypxl1 = ipart(yend) as i32;
        
        let a = fpart(yend);
        let b = 1.0 - a;
        
        if steep {
            draw(ypxl1,   xpxl1,  b * xgap);
            draw(ypxl1+1, xpxl1,  a * xgap);
        } else {
            draw(xpxl1, ypxl1,    b * xgap);
            draw(xpxl1, ypxl1+1,  a * xgap);
        }
        
        let mut intery = yend + gradient; // first y-intersection for the main loop
    
        // handle second endpoint
    
        let xend = round(x1);
        let yend = y1 + gradient * (xend - x1);
        let xgap = fpart(x1 + 0.5);
        let xpxl2 = xend as i32; //this will be used in the main loop
        let ypxl2 = ipart(yend) as i32;
        
        let a = fpart(yend);
        let b = 1.0 - a;
        
        if steep {
            draw(ypxl2,   xpxl2,  b * xgap);
            draw(ypxl2+1, xpxl2,  a * xgap);
        } else {
            draw(xpxl2, ypxl2,   b * xgap);
            draw(xpxl2, ypxl2+1, a * xgap);
        }
        
        // main loop
    
        if steep {
            for x in xpxl1 + 1 .. xpxl2 {
                let a = fpart(intery);
                let py = ipart(intery) as i32;
                
                draw(py,   x, 1.0 - a);
                draw(py+1, x, a);
                intery += gradient;
            }
        } else {
            for x in xpxl1 + 1 .. xpxl2 {
                let a = fpart(intery);
                let py = ipart(intery) as i32;
                
                draw(x, py,   1.0 - a);
                draw(x, py+1, a);
                intery += gradient;
            }
        }
    }
}


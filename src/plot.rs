use canvas::Canvas;
use rand;
use rand::distributions::{IndependentSample, Range as Uniform};
use nalgebra::{Vector2, Point2};
use num::{Float, cast};
use std::ops::Range;
use std::rc::Rc;
use std::convert::{From, Into};
use pen::Pen;

pub type N = f64;

pub trait SampleItem {
    fn sample(&self, rng: &mut rand::ThreadRng) -> Point2<N>;
}
pub trait TracedItem {
    fn trace<'a>(&'a self) -> Box<Iterator<Item=Vector2<N>> + 'a>;
}
pub enum Item {
    Sampled(Box<SampleItem>, usize),
    Traced(Box<TracedItem>, usize, f64)
}

pub struct XY {
    f: Box<Fn(N) -> N>,
    range: Uniform<N>,
}
impl XY {
    pub fn new(f: Box<Fn(N) -> N>, range: Range<N>) -> XY
    {
        XY {
            f: f,
            range: Uniform::new(range.start, range.end),
        }
    }
}
impl SampleItem for XY {
    fn sample(&self, rng: &mut rand::ThreadRng) -> Point2<N> {
        let x = self.range.ind_sample(rng);
        Point2::new(x, (self.f)(x))
    }
}

pub struct Parametric  {
    f: Box<Fn(N) -> Point2<N>>,
    range: Uniform<N>
}
impl Parametric {
    pub fn new(f: Box<Fn(N) -> Point2<N>>, range: Range<N>) -> Parametric
    {
        Parametric {
            f: f,
            range: Uniform::new(range.start, range.end)
        }
    }
}
impl SampleItem for Parametric {
    fn sample(&self, rng: &mut rand::ThreadRng) -> Point2<N> {
        let c = self.range.ind_sample(rng);
        (self.f)(c)
    }
}

pub struct Figure {
    domain: Vector2<Range<N>>,
    items: Vec<Item>
}

impl Figure {
    pub fn new(x: Range<N>, y: Range<N>) -> Figure {
        Figure {
            domain: Vector2::new(x, y),
            items: Vec::new()
        }
    }

    pub fn sample<S>(&mut self, item: S, samples: usize) -> &mut Figure
    where S: SampleItem + 'static
    {
        self.items.push(Item::Sampled(box item, samples));
        self
    }
    
    pub fn trace<S>(&mut self, item: S, iterations: usize, strength: f64) -> &mut Figure
    where S: TracedItem + 'static
    {
        self.items.push(Item::Traced(box item, iterations, strength));
        self
    }

    /** do the actutal plotting on a canvas of the given size **/
    pub fn draw<C: Canvas>(&self, width: u32, height: u32) -> C {
        let mut canvas = C::empty(width, height);
        self.draw_on(&mut canvas);
        canvas
    }
    
    pub fn draw_on<C: Canvas>(&self, canvas: &mut C) {
        let offset = Vector2::new(self.domain.x.start, self.domain.y.start);
        let domain_size = Vector2::new(
            self.domain.x.end - self.domain.x.start,
            self.domain.y.end - self.domain.y.start
        );
        
        let ref mut rng = rand::thread_rng();
        let uniform01 = Uniform::new(0.0, 1.0);
        let (subpixel_width, subpixel_height) = canvas.subpixel_size();
        let (subpixel_width, subpixel_height) = (subpixel_width as i32, subpixel_height as i32);
        let canvas_scale: Vector2<N> = Vector2::<N>::new(
            subpixel_width.into(),
            subpixel_height.into()
        ) / domain_size;
        
        let clipped = |p: Vector2<i32>| -> Option<(u32, u32)> {
            if p.x >= 0 && p.x < subpixel_width && p.y >= 0 && p.y < subpixel_height {
                Some((p.x as u32, p.y as u32))
            } else {
                None
            }
        };
        
        for item in self.items.iter() {
            match *item {
                Item::Sampled(ref item, samples) => {
                    for n in 0 .. samples {
                        // p is in domain space
                        let p = item.sample(rng);
                        
                        // random offset
                        let noise = Vector2::new(
                            uniform01.ind_sample(rng),
                            uniform01.ind_sample(rng)
                        );
                        
                        let q = (p - offset).to_vector() * canvas_scale + noise;
                        let qx = Vector2::new(q.x as i32, q.y as i32);
                        if let Some((x, y)) = clipped(qx) {
                            canvas.put_sample(x, y);
                        }
                    }
                },
                Item::Traced(ref item, iterations, strength) => {
                    let mut pen = Pen::new(|p, v| {
                        if let Some((x, y)) = clipped(p) {
                            canvas.put_weighted_sample(x, y, strength * v);
                        }
                    });
                    let mut points = item.trace()
                    .map(|p| (p - offset) * canvas_scale);
                    
                    // initial position
                    if let Some(p) = points.next() {
                        pen.move_to(p);
                    }
                    
                    for p in points.take(iterations) {
                        pen.line_to(p);
                    }
                }
            }
        }
    }
}

#[test]
fn test_plot() {
    Figure::new(-5.0 .. 5.0, -5.0 .. 5.0)
        .sample(XY::new(Box::new(|x: f64| (1.0/x).sin()), -5.0 .. 5.0), 10_000);
}

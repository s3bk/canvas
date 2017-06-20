use canvas::{Canvas, Meta, Data, Initial};
use rand;
use std::ops::{Range, Add};
use std::fmt::Debug;
use pen::Pen;
use tuple::*;
use math::prelude::*;

pub trait SampleItem<N: Real> {
    fn sample(&self, rng: &mut rand::ThreadRng) -> T2<N, N>;
}
pub trait TracedItem<N: Real> {
    fn trace<'a>(&'a self) -> Box<Iterator<Item=T2<N, N>> + 'a>;
}

#[derive(Copy, Clone)]
pub enum LineStyle {
    Line,
    Points
}

pub enum Item<N: Real> {
    /// the second argument is the number of samples to draw
    Sampled(Box<SampleItem<N>>, usize),

    /// the second argument is the number of points to draw
    /// the third argument is the strength to draw with (line intensity)
    /// forth argument 
    Traced(Box<TracedItem<N>>, usize)
}

pub struct XY<N> {
    f: Box<Fn(N) -> N>,
    offset: N,
    scale: N
}
impl<N: Real> XY<N> {
    pub fn new(f: Box<Fn(N) -> N>, range: Range<N>) -> XY<N>
    {
        XY {
            f: f,
            offset: range.start,
            scale:  range.end - range.start
        }
    }
}
impl<N: Real> SampleItem<N> for XY<N> {
    fn sample(&self, rng: &mut rand::ThreadRng) -> T2<N, N> {
        let x = N::uniform01(rng) * self.scale + self.offset;
        T2(x.clone(), (self.f)(x))
    }
}

pub struct Parametric<N>  {
    f: Box<Fn(N) -> T2<N, N>>,
    offset: N,
    scale: N
}
impl<N: Real> Parametric<N> {
    pub fn new(f: Box<Fn(N) -> T2<N, N>>, range: Range<N>) -> Parametric<N>
    {
        Parametric {
            f: f,
            offset: range.start,
            scale:  range.end - range.start
        }
    }
}
impl<N: Real> SampleItem<N> for Parametric<N> {
    fn sample(&self, rng: &mut rand::ThreadRng) -> T2<N, N> {
        let c = N::uniform01(rng) * self.scale + self.offset;
        (self.f)(c)
    }
}

pub struct Figure<N: Real> {
    offset: T2<N, N>,
    size:   T2<N, N>,
    items:  Vec<Item<N>>
}

impl<N> Figure<N> where N: Real
    + Cast<isize> + Cast<usize>,
    usize: Cast<N>, isize: Cast<N>
{
    pub fn new(x: Range<N>, y: Range<N>) -> Figure<N> {
        Figure {
            offset: T2(x.start, y.start),
            size:   T2(x.end - x.start, y.end - y.start),
            items:  Vec::new()
        }
    }

    pub fn sample<S>(&mut self, item: S, samples: usize) -> &mut Figure<N>
    where S: SampleItem<N> + 'static
    {
        self.items.push(Item::Sampled(Box::new(item), samples));
        self
    }

    pub fn trace<S>(&mut self, item: S, iterations: usize)
     -> &mut Figure<N> where S: TracedItem<N> + 'static
    {
        self.items.push(Item::Traced(Box::new(item), iterations));
        self
    }

    /** do the actutal plotting on a canvas of the given size **/
    pub fn draw<C>(&self, width: usize, height: usize) -> C where
        C: Canvas, C::Data: Initial, <C::Data as Data>::Item: Real
    {
        let mut canvas = C::new(
            C::Meta::new(width, height),
            C::Data::initial(width, height)
        );
        self.draw_on(&mut canvas);
        canvas
    }
    
    pub fn draw_on<C>(&self, canvas: &mut C)
    where C: Canvas, <C::Data as Data>::Item: Real
    {
        let ref mut rng = rand::thread_rng();
        canvas.run_mut(|meta, data| {
            let (subpixel_width, subpixel_height) = meta.subpixel_size();
            let subpixel_size: T2<N, N> = T2(subpixel_width, subpixel_height).cast().unwrap();
            let canvas_scale: T2<N, N> = subpixel_size / self.size;

            for item in self.items.iter() {
                match *item {
                    Item::Sampled(ref item, samples) => data.apply(
                        (0 .. samples).map(|_| {
                            let p = item.sample(rng);
                            (p - self.offset) * canvas_scale + T2::uniform01(rng)
                        })
                        .filter_map(|p: T2<N, N>| p.clip(T2(0, 0) ... T2(subpixel_width-1, subpixel_height-1)))
                        .map(|T2(x, y)| (meta.index((x, y)), Real::int(1))),
                        
                        |v, increment| v + increment
                    ),
                    Item::Traced(ref item, iterations) => {
                        data.apply(
                            item.trace()
                            .take(iterations)
                            .map(|p| (p - self.offset) * canvas_scale)
                            .map(|p| p + T2::uniform01(rng))
                            .filter_map(|p| p.cast())
                            //.filter_map(|p: T2<N, N>| p.clip(T2(0, 0) ... T2(subpixel_width-1, subpixel_height-1)))
                            .map(|T2(x, y)| (meta.index((x, y)), Real::int(1))),
                            
                            |v, increment| v + increment
                        );
                    }
                }
            }
        });
    }
}

#[test]
fn test_plot() {
    Figure::new(-5.0 .. 5.0, -5.0 .. 5.0)
        .sample(XY::new(Box::new(|x: f32| (1.0/x).sin()), -5.0 .. 5.0), 10_000);
}

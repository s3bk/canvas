use canvas::{Canvas, Meta, Data, Initial};
use rand;
use rand::distributions::{IndependentSample, Range as Uniform};
use rand::distributions::range::SampleRange;
use std::ops::{Range, AddAssign};
use std::fmt::Debug;
use pen::Pen;
use num::{cast, NumCast, Num, Float};
use tuple::*;

pub trait SampleItem<N: Num> {
    fn sample(&self, rng: &mut rand::ThreadRng) -> T2<N, N>;
}
pub trait TracedItem<N: Num> {
    fn trace<'a>(&'a self) -> Box<Iterator<Item=T2<N, N>> + 'a>;
}
pub enum Item<N: Num> {
    /// the second argument is the number of samples to draw
    Sampled(Box<SampleItem<N>>, usize),

    /// the second argument is the number of points to draw
    /// the third argument is the strength to draw with (line intensity)
    Traced(Box<TracedItem<N>>, usize, f32)
}

pub struct XY<N> {
    f: Box<Fn(N) -> N>,
    range: Uniform<N>,
}
impl<N: Num + SampleRange + PartialOrd> XY<N> {
    pub fn new(f: Box<Fn(N) -> N>, range: Range<N>) -> XY<N>
    {
        XY {
            f: f,
            range: Uniform::new(range.start, range.end),
        }
    }
}
impl<N: Num + SampleRange + Clone> SampleItem<N> for XY<N> {
    fn sample(&self, rng: &mut rand::ThreadRng) -> T2<N, N> {
        let x = self.range.ind_sample(rng);
        T2(x.clone(), (self.f)(x))
    }
}

pub struct Parametric<N>  {
    f: Box<Fn(N) -> T2<N, N>>,
    range: Uniform<N>
}
impl<N: Num + SampleRange + PartialOrd> Parametric<N> {
    pub fn new(f: Box<Fn(N) -> T2<N, N>>, range: Range<N>) -> Parametric<N>
    {
        Parametric {
            f: f,
            range: Uniform::new(range.start, range.end)
        }
    }
}
impl<N: Num + SampleRange> SampleItem<N> for Parametric<N> {
    fn sample(&self, rng: &mut rand::ThreadRng) -> T2<N, N> {
        let c = self.range.ind_sample(rng);
        (self.f)(c)
    }
}

pub struct Figure<N: Num> {
    offset: T2<N, N>,
    size:   T2<N, N>,
    items:  Vec<Item<N>>
}

impl<N> Figure<N> where
    N: Float + NumCast + SampleRange + PartialOrd + Clone + AddAssign + Debug
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

    pub fn trace<S>(&mut self, item: S, iterations: usize, strength: f32) -> &mut Figure<N>
    where S: TracedItem<N> + 'static
    {
        self.items.push(Item::Traced(Box::new(item), iterations, strength));
        self
    }

    /** do the actutal plotting on a canvas of the given size **/
    pub fn draw<C>(&self, width: usize, height: usize) -> C where
        C: Canvas, C::Data: Initial, <C::Data as Data>::Item: NumCast + AddAssign
    {
        let mut canvas = C::new(
            C::Meta::new(width, height),
            C::Data::initial(width, height)
        );
        self.draw_on(&mut canvas);
        canvas
    }

    pub fn draw_on<C>(&self, canvas: &mut C)
    where C: Canvas, <C::Data as Data>::Item: NumCast + AddAssign
    {
        canvas.run_mut(|meta, data| {
            let ref mut rng = rand::thread_rng();
            let uniform01 = Uniform::new(cast(0u8).unwrap(), cast(1u8).unwrap());
            let (subpixel_width, subpixel_height) = meta.subpixel_size();
            let subpixel_size: T2<N, N> = T2(
                cast(subpixel_width).unwrap(),
                cast(subpixel_height).unwrap()
            );
            let canvas_scale: T2<N, N> = subpixel_size / self.size;

            #[inline]
            let clipped = |p: T2<isize, isize>| -> Option<(usize, usize)> {
                if p.0 >= 0 && p.0 < subpixel_width as isize
                && p.1 >= 0 && p.1 < subpixel_height as isize {
                    Some((p.0 as usize, p.1 as usize))
                } else {
                    None
                }
            };

            for item in self.items.iter() {
                match *item {
                    Item::Sampled(ref item, samples) => {
                        data.apply(
                            (0 .. samples)
                            .map(|_| {
                                // p is in domain space
                                let p = item.sample(rng);

                                // random offset
                                let noise: T2<N, N> = T2(
                                    uniform01.ind_sample(rng),
                                    uniform01.ind_sample(rng)
                                );
                            
                                (p - self.offset) * canvas_scale + noise
                            })
                            .filter_map(|p| {
                                if let (Some(x), Some(y)) = (cast(p.0), cast(p.1)) {
                                    Some(T2(x, y))
                                } else {
                                    None
                                }
                            })
                            .filter_map(|p| clipped(p))
                            .map(|(x, y)| (meta.index((x, y)), cast(1u8).unwrap())),
                            |v, increment| {
                                *v += increment;
                            }
                        );
                    },
                    Item::Traced(ref item, iterations, strength) => {
                        let mut points = item.trace()
                        .map(|p| (p - self.offset) * canvas_scale);

                        let mut pen = Pen::new(|p, v| {
                            if let Some((x, y)) = clipped(p) {
                                *data.get_mut(meta.index((x, y)))
                                 += cast(strength * v).unwrap();
                            }
                        });
                        
                        if let Some(mut start) = points.next() {
                            for end in points.take(iterations) {
                                pen.line(start, end);
                                start = end;
                            }
                        }
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

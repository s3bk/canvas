use canvas::{Canvas, Meta, Data, Initial};
use std::ops::{Range};
use tuple::T2;
use math::prelude::*;
use contour::ContourPlot;
use pen::Pen;
use colormap;
use image::GrayImage;
use array::{Array, RowMajor};
use rng::{VRng, DefaultRng};

pub struct Figure<N: Real = f32, C: Canvas = Array<Vec<f32>, RowMajor>> {
    offset: T2<N, N>,
    size:   T2<N, N>,
    canvas: C
}

impl<N, C> Figure<N, C>
    where N: Real + Cast<isize> + Cast<usize> + Copy,
          C: Canvas,
          usize: Cast<N>, isize: Cast<N>,
          u8: Cast<<C::Data as Data>::Item>
{
    pub fn new(x: Range<N>, y: Range<N>, (width, height): (usize, usize)) -> Figure<N, C>
        where C::Data: Initial
    {
        Figure {
            offset: T2(x.start, y.start),
            size:   T2(x.end - x.start, y.end - y.start),
            canvas: C::new(
                C::Meta::new(width, height),
                C::Data::initial(width, height)
            )
        }
    }

    pub fn sample_xy<F, V>(&mut self, samples: usize, mut func: F) -> &mut Self
        where F: FnMut(V) -> V, V: Real<Scalar=N>
    {
        let mut rng = DefaultRng::new();
        let offset = self.offset.0;
        let scale = self.size.0;
        self.sample(samples, || {
            let x = V::splat(offset) + V::splat(scale) * rng.next();
            T2(x, func(x))
        })
    }
    
    pub fn sample<F, V>(&mut self, samples: usize, mut func: F) -> &mut Self
        where F: FnMut() -> T2<V, V>, V: Real<Scalar=N>
    {
        let size = self.size;
        let offset = self.offset;
        let mut rng = DefaultRng::new();

        self.canvas.run_mut(|meta, data| {
            let (subpixel_width, subpixel_height) = meta.subpixel_size();
            let subpixel_size: T2<N, N> = T2(subpixel_width, subpixel_height).cast().unwrap();
            let canvas_scale: T2<N, N> = subpixel_size / size;

            for _ in 0 .. samples {
                let r: T2<V, V> = rng.next();
                let s: T2<V, V> = (func() - offset.map(Real::splat)) * canvas_scale.map(Real::splat) + r;
                for e in s.map(Real::values) {
                    let e: T2<N, N> = e;
                    if let Some(T2(x, y)) = e.cast_clipped(T2(0, 0) ... T2(subpixel_width-1, subpixel_height-1)) {
                        *data.get_mut(meta.index((x, y))) += 1u8.cast().unwrap();
                    }
                }
            }
        });
        
        self
    }

    pub fn trace<I>(&mut self, iter: I, iterations: usize) -> &mut Self
        where I: Iterator<Item=T2<N, N>>
    {
        let mut rng = DefaultRng::new();
        let size = self.size;
        let offset = self.offset;
        self.canvas.run_mut(|meta, data| {
            let (subpixel_width, subpixel_height) = meta.subpixel_size();
            let subpixel_size: T2<N, N> = T2(subpixel_width, subpixel_height).cast().unwrap();
            let canvas_scale: T2<N, N> = subpixel_size / size;

            data.apply(
                iter.take(iterations)
                    .map(|p| (p - offset) * canvas_scale)
                    .map(|p| {
                        let r: T2<N, N> = rng.next();
                        p + r
                    })
                    .filter_map(|p: T2<N, N>| p.cast_clipped(T2(0, 0) ... T2(subpixel_width-1, subpixel_height-1)))
                    .map(|T2(x, y)| (meta.index((x, y)), (1u8).cast().unwrap())),
                
                |v, increment| v + increment
            );
        });
        
        self
    }

    pub fn contour<F>(&mut self, func: F) -> &mut Self
        where F: Fn(T2<N, N>) -> N,
              N: Real<Bool=bool> + Cast<f32>,
              f32: Cast<<C::Data as Data>::Item>
    {
        let size = self.size;
        let offset = self.offset;
        
        self.canvas.run_mut(|meta, data| {
            let (subpixel_width, subpixel_height) = meta.subpixel_size();
            let subpixel_size: T2<N, N> = T2(subpixel_width, subpixel_height).cast().unwrap();

            let (w, h) = meta.size();
            let n = 9; //(w as f32).log(2.0).ceil() as u8;
            let scale_inv = size / subpixel_size;

            let start: T2<usize, usize> = T2(0, 0);
            let end: T2<usize, usize> = T2(w-1, h-1);
            
            let mut c = ContourPlot {
                plot_depth: n,
                search_depth: n/2,
                func: |p: T2<usize, usize>| {
                    let p: T2<N, N> = p.cast().unwrap();
                    func(p * scale_inv + offset)
                },
                pen: Pen::new(|p: T2<isize, isize>, v| {
                    let op: Option<T2<usize, usize>> = p.cast_clipped(start ... end);
                    if let Some(T2(x, y)) = op {
                        let pixel = data.get_mut(meta.index((x, y)));
                        *pixel += v.cast().unwrap();
                    }
                })
            };
            c.run();
        });
        
        self
    }

    #[inline]
    pub fn contour_gradient<F, V, R>(&mut self, func: F, samples: usize, iterations: usize, rng: &mut R) -> &mut Self
        where F: Fn(T2<V, V>) -> (V, T2<V, V>), V: Real<Scalar=N>, R: VRng<T2<V, V>>
    {
        let size = self.size;
        let offset = self.offset;

        self.canvas.run_mut(|meta, data| {
            let (subpixel_width, subpixel_height) = meta.subpixel_size();
            let subpixel_size: T2<N, N> = T2(subpixel_width, subpixel_height).cast().unwrap();
            let canvas_scale: T2<N, N> = subpixel_size / size;

            for sample_nr in 0 .. samples {
                let r: T2<V, V> = rng.next();
                let mut p = r * size.map(Real::splat) + offset.map(Real::splat); // random point on the canvas
                for _ in 0 .. iterations {
                    let (q, grad_inv) = func(p);
                    p = p - grad_inv * q; // * T2::uniform01(rng);
                }
                
                let r: T2<V, V> = rng.next();
                let s: T2<V, V> = (p - offset.map(Real::splat)) * canvas_scale.map(Real::splat) + r;

                for e in s.map(Real::values) {
                    let e: T2<N, N> = e;
                    if let Some(T2(x, y)) = e.cast_clipped(T2(0, 0) ... T2(subpixel_width-1, subpixel_height-1)) {
                        *data.get_mut(meta.index((x, y))) += 1u8.cast().unwrap();
                    }
                }
            }
        });

        self
    }

    pub fn grayscale(&self, max: Option<<C::Data as Data>::Item>) -> GrayImage
        where <C::Data as Data>::Item: Real<Bool=bool> + Copy + Cast<usize>
    {
        colormap::grayscale(&self.canvas, max)
    }

}

use image::{Rgba, RgbaImage, Luma, GrayImage};
use std::f32::consts::PI;
use std::cmp;
use palette::{Lch, LabHue, IntoColor, Gradient};
use num::{ToPrimitive};
use canvas::{Canvas, Meta, Data};

pub trait ColorMap {
    fn build(&self, steps: usize) -> Vec<Rgba<u8>>;
}

lazy_static! {
    pub static ref MAP_COLORFUL: [(f32, Lch); 5] = [
        (0.0, Lch::new(0.0, 1., LabHue::from_radians(-2. * PI / 3.))),
        (0.3, Lch::new(0.2, 1., LabHue::from_radians(-1. * PI / 3.))),
        (0.6, Lch::new(0.5, 1., LabHue::from_radians(0.))),
        (0.8, Lch::new(0.8, 1., LabHue::from_radians(PI / 3.))),
        (1.0, Lch::new(1.0, 0., LabHue::from_radians(PI / 3.)))
    ];
}
impl<'a> ColorMap for &'a [(f32, Lch)] {
    fn build(&self, steps: usize) -> Vec<Rgba<u8>> {
        let k = steps as f32;
        let gradient = Gradient::with_domain(self.to_vec());
        
        (0 .. steps).map(|i| {
            let (r, g, b) = gradient.get(i as f32 / k).into_rgb().to_pixel();
            Rgba([r, g, b, 255])
        }).collect()
    }
}

pub fn map<C: Canvas>(canvas: &C, colormap: &ColorMap) -> RgbaImage
    where <C::Data as Data>::Item: ToPrimitive
{
    canvas.run(|meta, data| {
        // figure out max value
        let (width, height) = meta.size();
        let mut max_value = 0.0f32;
        for y in 0 .. height {
            for x in 0 .. width {
                if let Some(v) = data.get(meta.index((x, y))).to_f32() {
                    if v > max_value {
                        max_value = v;
                    }
                }
            }
        }
        
        println!("{}", max_value);
        let mut imgbuf = RgbaImage::new(width as u32, height as u32);
        if max_value == 0. {
            return imgbuf;
        }
        
        let steps = 1024;
        let cmap = colormap.build(steps);
        let scale = (steps - 1) as f32 / max_value;
        
        for (x, y, p) in imgbuf.enumerate_pixels_mut() {
            let idx = meta.index((x as usize, y as usize));
            let v = data.get(idx)
            .to_f32()
            .map(|v| v * scale)
            .unwrap_or(0.);
            *p = cmap[cmp::min(v as usize, steps-1)];
        }
        
        imgbuf
    })
}

pub fn grayscale<C: Canvas>(canvas: &C) -> GrayImage
    where <C::Data as Data>::Item: ToPrimitive
{
    canvas.run(|meta, data| {
        // figure out max value
        let (width, height) = meta.size();
        let mut max_value = 0.0f32;
        for y in 0 .. height {
            for x in 0 .. width {
                if let Some(v) = data.get(meta.index((x, y))).to_f32() {
                    if v > max_value {
                        max_value = v;
                    }
                }
            }
        }
        
        println!("{}", max_value);
        let mut imgbuf = GrayImage::new(width as u32, height as u32);
        if max_value == 0. {
            return imgbuf;
        }
        
        let steps = 256;
        let scale = (steps - 1) as f32 / max_value;
        
        for (x, y, p) in imgbuf.enumerate_pixels_mut() {
            let idx = meta.index((x as usize, y as usize));
            let v = data.get(idx)
            .to_f32()
            .map(|v| v * scale)
            .unwrap_or(0.);
            *p = Luma { data: [255 - v as u8] };
        }
        
        imgbuf
    })
}

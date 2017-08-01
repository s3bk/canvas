use image::{Rgba, RgbaImage, Luma, GrayImage};
use std::f32::consts::PI;
use palette::{Lch, LabHue, IntoColor, Gradient};
use canvas::{Canvas, Meta, Data};
use math::real::Real;
use math::cast::Cast;

pub trait ColorMap: Sync {
    fn build(&self, steps: usize) -> Vec<Rgba<u8>>;
}

lazy_static! {
    pub static ref MAP_COLORFUL: Vec<(f32, Lch)> = vec![
        (0.0, Lch::new(0.0, 1., LabHue::from_radians(-2. * PI / 3.))),
        (0.3, Lch::new(0.2, 1., LabHue::from_radians(-1. * PI / 3.))),
        (0.6, Lch::new(0.5, 1., LabHue::from_radians(0.))),
        (0.9, Lch::new(0.8, 1., LabHue::from_radians(PI / 3.))),
        (1.0, Lch::new(1.0, 0., LabHue::from_radians(PI / 3.)))
    ];
    pub static ref MAP_STEEL: Vec<(f32, Lch)> = vec![
        (0.0, Lch::new(1.0, 0.0, LabHue::from_radians(-2. * PI / 3.))),
        (0.1, Lch::new(0.9, 0.1, LabHue::from_radians(-2. * PI / 3.))),
        (0.2, Lch::new(0.8, 0.2, LabHue::from_radians(-2. * PI / 3.))),
        (1.0, Lch::new(0.0, 0.5, LabHue::from_radians(-2. * PI / 3.))),
    ];
}
impl ColorMap for Vec<(f32, Lch)> {
    fn build(&self, steps: usize) -> Vec<Rgba<u8>> {
        let k = steps as f32;
        let gradient = Gradient::with_domain(self.to_vec());
        
        (0 .. steps).map(|i| {
            let (r, g, b) = gradient.get(i as f32 / k).into_rgb().to_pixel();
            Rgba([r, g, b, 255])
        }).collect()
    }
}

pub fn map<C, M>(canvas: &C, colormap: &M) -> RgbaImage
    where C: Canvas, M: ColorMap, <C::Data as Data>::Item: Real<Bool=bool> + Copy + Cast<usize>
{
    let (width, height) = canvas.run(|meta, _| meta.size());
    let mut imgbuf = RgbaImage::new(width as u32, height as u32);
    map_to(canvas, colormap, &mut imgbuf);
    imgbuf
}

fn map_to_index<C, F, O>(canvas: &C, steps: usize, max: Option<<C::Data as Data>::Item>, f: F) -> O
    where C: Canvas, <C::Data as Data>::Item: Real<Bool=bool> + Copy + Cast<usize>, F: FnOnce((u32, u32), &Fn(u32, u32) -> usize) -> O
{
    canvas.run(|meta, data| {
        // figure out max value
        let (width, height) = meta.size();
        let max_value = max.unwrap_or_else(|| {
            let mut max_value = <<C::Data as Data>::Item as Real>::int(0);
            for y in 0 .. height {
                for x in 0 .. width {
                    let &v = data.get(meta.index((x, y)));
                    if v.gt(max_value) {
                        max_value = v;
                    }
                }
            }
            max_value
        });
        
        let scale = <<C::Data as Data>::Item as Real>::int(steps as i16 - 1) / max_value.sqrt();
        
        f((width as u32, height as u32), &|x, y| {
            let idx = meta.index((x as usize, height - 1 - y as usize));
            let v = *data.get(idx);
            (v.sqrt() * scale).cast_clamped(0 ... steps-1)
        })
    })
}
pub fn map_to<C, M>(canvas: &C, colormap: &M, imgbuf: &mut RgbaImage)
    where C: Canvas, M: ColorMap, <C::Data as Data>::Item: Real<Bool=bool> + Copy + Cast<usize>
{
    let steps = 1024;
    let cmap = colormap.build(steps);
    map_to_index(canvas, steps, None, |_, get| {        
        for (x, y, p) in imgbuf.enumerate_pixels_mut() {
            *p = cmap[get(x, y)];
        }
    });
}

pub fn grayscale<C: Canvas>(canvas: &C, max: Option<<C::Data as Data>::Item>) -> GrayImage
    where <C::Data as Data>::Item: Real<Bool=bool> + Copy + Cast<usize>
{
    map_to_index(canvas, 256, max, |(width, height), get| {
        let mut imgbuf = GrayImage::new(width as u32, height as u32);
        for (x, y, p) in imgbuf.enumerate_pixels_mut() {
            *p = Luma { data: [255 - get(x, y) as u8] };
        }
        
        imgbuf
    })
}

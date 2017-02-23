use canvas::Canvas;
use image;

impl Canvas for image::GrayImage {
    fn pixel_size(&self) -> (u32, u32) {
        (self.width(), self.height())
    }
    fn subpixel_size(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    /** sx, sy are is in subpixel coordinates **/
    fn put_sample(&mut self, x: u32, y: u32) {
        let ref mut p = self.get_pixel_mut(x, y);
        p.data[0] = p.data[0].saturating_sub(1);
    }
    
    fn empty(width: u32, height: u32) -> image::GrayImage {
        image::GrayImage::from_pixel(
            width,
            height,
            image::Luma{ data:[255] }
        )
    }
}

impl Canvas for image::RgbImage {
    fn pixel_size(&self) -> (u32, u32) {
        (self.width(), self.height())
    }
    fn subpixel_size(&self) -> (u32, u32) {
        (3 * self.width(), self.height())
    }

    /** sx, sy are is in subpixel coordinates **/
    fn put_sample(&mut self, sx: u32, y: u32) {
        for i in 0..3 {
            let x = (sx+i) / 3;
            let c = ((sx + i) % 3) as usize;
            let ref mut p = self.get_pixel_mut(x, y);
            p.data[c] = p.data[c].saturating_sub(1);
        }
    }
    
    fn empty(width: u32, height: u32) -> image::RgbImage {
        image::RgbImage::from_pixel(
            width,
            height,
            image::Rgb{ data:[255, 255, 255] }
        )
    }
}

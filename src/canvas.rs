pub trait Canvas {
    /** get the size of one pixel **/
    fn pixel_size(&self) -> (u32, u32);
    
    /** get the size of one subpixel. this may or may not be the same as pixel_size.
        defaults to pixel_size. **/
    fn subpixel_size(&self) -> (u32, u32) {
        self.pixel_size()
    }
    
    /** add a sample point to the canvas. no anti-aliasing is done. **/
    /** p is in subpixel coordinates **/
    fn put_sample(&mut self, sx: u32, sy:u32);
    
    fn empty(width: u32, height: u32) -> Self;
}

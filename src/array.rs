use std::{cmp};
use std::ops::{Index, IndexMut};
use canvas::{Canvas, Data, Meta, Initial};
use image::GrayImage;

#[derive(Copy, Clone)]
enum Square {
    A,
    B,
    C,
    D
}

/*
http://blog.notdot.net/2009/11/Damn-Cool-Algorithms-Spatial-indexing-with-Quadtrees-and-Hilbert-Curves

    A       B       C       D

    0D 3B   2B 3A   2C 1C   0A 1D
    1A 2A   1B 0C   3D 0B   3C 2D
*/

use self::Square::*;
static HILBERT_MAP: [[(u8, Square); 4]; 4] = [
    [(0, D), (1, A), (3, B), (2, A)],
    [(2, B), (1, B), (3, A), (0, C)],
    [(2, C), (3, D), (1, C), (0, B)],
    [(0, A), (3, C), (1, D), (2, D)]
];

#[inline]
fn hilbert_index(x: usize, y: usize, order: u8) -> usize {
    let mut square = A;
    let mut position = 0usize;
    
    for i in 0 .. order as usize {
        let idx = (((x >> i) & 1) << 1) | ((y >> i) & 1);
        let (quadrant, new_square) = HILBERT_MAP[square as usize][idx];
        position |= (quadrant as usize) << (2 * i);
        square = new_square;
    }
    position
}

#[test]
fn test_hilbert_index() {
    assert_eq!(hilbert_index(5, 2, 3), 55);
    
}

pub struct Mapped2d {
    block_size: usize,
    mask_x:     usize,
    mask_y:     usize,
    order:      u8,
    shift_x:    u8,
    shift_y:    u8
}

impl Meta for Mapped2d {
    fn new(width: usize, height: usize) -> Mapped2d {
        assert!(width.is_power_of_two());
        assert!(height.is_power_of_two());
        
        let w_pow2 = width.trailing_zeros();
        let h_pow2 = height.trailing_zeros();
        
        let order = cmp::min(w_pow2, h_pow2);
        let block_size = (2usize).pow(cmp::max(w_pow2, h_pow2) - order);
        
        Mapped2d {
            order:      order as u8,
            block_size: block_size,
            shift_x:    (w_pow2 - order) as u8,
            shift_y:    (h_pow2 - order) as u8,
            mask_x:     (1 << (w_pow2 - order)) - 1,
            mask_y:     (1 << (h_pow2 - order)) - 1
        }
    }
    
    #[inline(always)]
    fn size(&self) -> (usize, usize) {
        (
            (2usize).pow((self.order + self.shift_x) as u32),
            (2usize).pow((self.order + self.shift_y) as u32)
        )
    }
    
    #[inline(always)]
    fn index(&self, p: (usize, usize)) -> usize {
        let index = hilbert_index(
            p.0 as usize >> self.shift_x,
            p.1 as usize >> self.shift_y,
            self.order
        );
        
        // offset into block
        let offset_x = p.0 & self.mask_x;
        let offset_y = p.1 & self.mask_y;
        let offset = (2usize).pow(self.shift_x as u32) * offset_y + offset_x;
        
        index * self.block_size + offset
    }
}

#[test]
fn test_map() {
    use tuple::T2;
    use canvas::Initial;
    
    let width = 2048;
    let height = 1024;
    
    let data = vec![T2(0u16, 0u16); width * height];
    let mut map = Array::new(
        Mapped2d::new(width, height),
        Vec::<T2<u16, u16>>::initial(width, height)
    );
    
    for x in 0 .. width {
        for y in 0 .. height {
            map[(x, y)] = T2(x as u16, y as u16);
        }
    }
    
    for x in 0 .. width {
        for y in 0 .. height {
            assert_eq!(map[(x, y)], T2(x as u16, y as u16));
        }
    }
}


pub struct RowMajor {
    width:  usize,
    height: usize
}
pub struct Array<D, M> {
    pub data:   D,
    pub meta:   M
}

impl<D, M> Canvas for Array<D, M> where D: Data, M: Meta
{
    type Data = D;
    type Meta = M;
    
    #[inline(always)]
    fn run<F, O>(&self, f: F) -> O
        where F: FnOnce(&M, &D) -> O
    {
        f(&self.meta, &self.data)
    }
    
    fn run_mut<F, O>(&mut self, f: F) -> O
        where F: FnOnce(&M, &mut D) -> O
    {
        f(&self.meta, &mut self.data)
    }
    
    fn new(meta: M, data: D) -> Self {
        Array {
            data:   data,
            meta:   meta
        }
    }
}

impl Meta for RowMajor {
    fn new(width: usize, height: usize) -> Self {
        RowMajor {
            width:  width,
            height: height
        }
    }

    #[inline(always)]
    fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    
    #[inline(always)]
    fn index(&self, p: (usize, usize)) -> usize {
        p.0 + p.1 * self.width
    }
}

impl Array<Vec<u8>, RowMajor> {
    pub fn to_image(self) -> GrayImage {
        let (width, height) = self.meta.size();
        GrayImage::from_raw(width as u32, height as u32, self.data).unwrap()
    }
}
impl<D, M> Index<(usize, usize)> for Array<D, M> where D: Data, M: Meta
{
    type Output = D::Item;
    
    #[inline(always)]
    fn index(&self, idx: (usize, usize)) -> &Self::Output {
        let index = self.meta.index(idx);
        self.data.get(index)
    }
}
impl<D, M> IndexMut<(usize, usize)> for Array<D, M> where D: Data, M: Meta
{
    #[inline(always)]
    fn index_mut(&mut self, idx: (usize, usize)) -> &mut Self::Output {
        let index = self.meta.index(idx);
        self.data.get_mut(index)
    }
}

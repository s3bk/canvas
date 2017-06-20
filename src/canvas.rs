use std::ops::{DerefMut};
use std::mem;

pub trait Meta {
    fn new(usize, usize) -> Self;

    fn size(&self) -> (usize, usize);
    
    /** get the size of one subpixel. this may or may not be the same as pixel_size.
        defaults to pixel_size. **/
    fn subpixel_size(&self) -> (usize, usize) {
        self.size()
    }
    
    fn index(&self, p: (usize, usize)) -> usize;
}

pub trait Data {
    type Item;
    
    fn map<F>(&mut self, f: F) where F: Fn(Self::Item) -> Self::Item;
        
    fn apply<I, F>(&mut self, it: I, f: F) where
        I: Iterator<Item=(usize, Self::Item)>, F: Fn(Self::Item, Self::Item) -> Self::Item;
        
    fn get(&self, index: usize) -> &Self::Item;
    fn get_mut(&mut self, index: usize) -> &mut Self::Item;
}

pub trait Canvas {
    type Data: Data;
    type Meta: Meta;
    
    fn run<F, O>(&self, f: F) -> O
        where F: FnOnce(&Self::Meta, &Self::Data) -> O;
        
    fn run_mut<F, O>(&mut self, f: F) -> O
        where F: FnOnce(&Self::Meta, &mut Self::Data) -> O;
    
    fn new(meta: Self::Meta, data: Self::Data) -> Self;
}
pub fn default<C>(width: usize, height: usize) -> C where
    C: Canvas, C::Data: Initial
{
    C::new(C::Meta::new(width, height), C::Data::initial(width, height))
}

pub trait Initial {
    fn initial(width: usize, height: usize) -> Self;
}

impl<A, T> Data for A where A: DerefMut<Target=[T]>, T: Default {
    type Item = T;
    
    fn map<F>(&mut self, f: F) where F: Fn(Self::Item) -> Self::Item {
        for v in self.iter_mut() {
            *v = f(mem::replace(v, T::default()));
        }
    }
    
    fn apply<I, F>(&mut self, it: I, f: F) where
        I: Iterator<Item=(usize, T)>, F: Fn(T, T) -> T
    {
        for (idx, v) in it {
            let data = &mut self[idx];
            *data = f(mem::replace(data, T::default()), v);
        }
    }
    
    fn get(&self, index: usize) -> &Self::Item {
        &self[index]
    }
    fn get_mut(&mut self, index: usize) -> &mut Self::Item {
        &mut self[index]
    }
}

impl<T> Initial for Vec<T> where T: Default {
    fn initial(width: usize, height: usize) -> Self {
        (0 .. (width * height)).map(|_| T::default()).collect()
    }
}

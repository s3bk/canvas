#![feature(conservative_impl_trait)]

extern crate rand;
extern crate num;
extern crate image;
extern crate tuple;
extern crate palette;
#[macro_use] extern crate lazy_static;

pub mod canvas;
pub mod plot;
pub mod array;
pub mod pen;
pub mod colormap;

pub use canvas::Canvas;

#![feature(conservative_impl_trait)]
#![feature(inclusive_range_syntax)]

extern crate rand;
extern crate image;
extern crate tuple;
extern crate palette;
extern crate math;
#[macro_use] extern crate lazy_static;

pub mod canvas;
pub mod plot;
pub mod array;
//pub mod pen;
//pub mod colormap;

pub use canvas::Canvas;

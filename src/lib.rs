#![feature(conservative_impl_trait)]
#![feature(inclusive_range_syntax)]
#![feature(associated_type_defaults)]
#![feature(thread_local)]

extern crate rand;
extern crate image;
extern crate tuple;
extern crate palette;
extern crate math;
#[macro_use] extern crate lazy_static;

pub mod canvas;
pub mod plot;
pub mod array;
pub mod contour;
pub mod pen;
pub mod colormap;

pub use canvas::Canvas;

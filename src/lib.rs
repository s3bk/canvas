#![feature(box_syntax)]

extern crate rand;
extern crate num;
extern crate nalgebra;
extern crate image;

pub mod canvas;
pub mod plot;
pub mod adapters;
pub mod integrate;
pub mod pen;

pub use canvas::Canvas;

#![feature(non_ascii_idents)]
#![feature(core_intrinsics)]
#![feature(conservative_impl_trait)]

#[macro_use]
extern crate gfx;
extern crate gfx_app;
extern crate gfx_core;
extern crate winit;

extern crate canvas;
extern crate tuple;
extern crate math;
extern crate fmath;

use canvas::plot::{LineStyle};
use tuple::T2;
use canvas::plot::{Figure};
use canvas::array::{Array, RowMajor};
use canvas::canvas::{Canvas, Data, Meta};
use math::integrate::Integration;
use math::real::Real;
use fmath::*;
use gfx::handle::Texture;

fn cos(x: f32) -> f32 {
    let x = f32x8::splat(x);
    poly8_f32x8_avx(POLY32_COS_8_PI, x * x).extract(0)
}

#[allow(non_snake_case)]
#[inline]
fn duffing(ɛ: f32, λ: f32, _Ω: f32, α: f32, β: f32)
 -> impl Fn(f32, T2<f32, f32>) -> T2<f32, f32>
{
    use std::intrinsics::{fmul_fast};
    move |t, s| {
        unsafe {
            T2(
                s.1,
                fmul_fast(ɛ, cos(t))
                - fmul_fast(λ, s.1)
                - fmul_fast(s.0, α + fmul_fast(fmul_fast(s.0, s.0), β))
            )
        }
    }
}

pub use gfx_app::ColorFormat;
pub use gfx::format::{Rgba8, DepthStencil};
use gfx::Bundle;
use gfx::texture;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    constant Locals {
        exp: f32 = "u_Exp",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        canvas: gfx::TextureSampler<u32> = "t_Canvas",
        exp: gfx::Global<f32> = "i_Exp",
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

impl Vertex {
    fn new(p: [f32; 2], u: [f32; 2]) -> Vertex {
        Vertex {
            pos: p,
            uv: u,
        }
    }
}

struct App<R: gfx::Resources>{
    bundle:     Bundle<R, pipe::Data<R>>,
    fig:        Figure<f32>,
    map:        Array<Vec<u32>, RowMajor>,
    tex:        Texture<R, gfx::format::R32>
}

impl<R: gfx::Resources> gfx_app::Application<R> for App<R> {
    fn new<F: gfx::Factory<R>>(factory: &mut F, backend: gfx_app::shade::Backend,
           window_targets: gfx_app::WindowTargets<R>) -> Self
    {
        use gfx::traits::FactoryExt;

        let width = 1024;
        let height = 1024;
        let map = Array::new(RowMajor::new(width, height), vec![0x3f000000; width * height]);
        let mut fig = Figure::new(-4.0f32 .. 4.0, -6.0 .. 6.0);
        fig.trace(
            Integration::new(
                duffing(7.5, 0.15, 1.0, 0.0, 1.0),
                T2(1.0, 1.0),
                0.0,
                1e-3
            ),
            200_000
        );
        
        let vs = gfx_app::shade::Source {
            glsl_120: include_bytes!("shader/blend_120.glslv"),
            glsl_150: include_bytes!("shader/blend_150.glslv"),
            .. gfx_app::shade::Source::empty()
        };
        let ps = gfx_app::shade::Source {
           // glsl_120: include_bytes!("shader/blend_120.glslf"),
            glsl_150: include_bytes!("shader/blend_150.glslf"),
            .. gfx_app::shade::Source::empty()
        };

        // fullscreen quad
        let vertex_data = [
            Vertex::new([-1.0, -1.0], [0.0, 1.0]),
            Vertex::new([ 1.0, -1.0], [1.0, 1.0]),
            Vertex::new([ 1.0,  1.0], [1.0, 0.0]),

            Vertex::new([-1.0, -1.0], [0.0, 1.0]),
            Vertex::new([ 1.0,  1.0], [1.0, 0.0]),
            Vertex::new([-1.0,  1.0], [0.0, 0.0]),
        ];
        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, ());

        let (width, height) = map.meta.size();
        let kind = texture::Kind::D2(
            width as texture::Size,
            height as texture::Size,
            texture::AaMode::Single
        );
        let tex = factory.create_texture::<gfx::format::R32>(
            kind, 1,
            gfx_core::memory::SHADER_RESOURCE,
            gfx::memory::Usage::Dynamic, Some(gfx::format::ChannelType::Uint)
        ).expect("create texture");
        let sampler = factory.create_sampler_linear();
        
        let pso = factory.create_pipeline_simple(
            vs.select(backend).expect("failed vertex shader"),
            ps.select(backend).expect("failed pixel shader"),
            pipe::new()
        ).expect("failed to crate pipeline");
        
        let cbuf = factory.create_constant_buffer(1);
        let canvas = factory.view_texture_as_shader_resource::<u32>(&tex, (0, 0), gfx::format::Swizzle::new()).expect("as shader ressource");
        
        let data = pipe::Data {
            vbuf:   vbuf,
            canvas: (canvas, sampler),
            exp:    1.,
            locals: cbuf,
            out:    window_targets.color,
        };

        App {
            bundle: Bundle::new(slice, pso, data),
            map:    map,
            fig:    fig,
            tex:    tex
        }
    }

    fn render<C: gfx::CommandBuffer<R>>(&mut self, encoder: &mut gfx::Encoder<R, C>) {
        //self.fig.draw_on(&mut self.map);
        
        let view_info = self.tex.get_info().to_image_info(0);
        encoder.update_texture::<gfx::format::R32, u32>(&self.tex, None, view_info, &self.map.data).expect("update texture");
        self.bundle.data.exp = 1.0;
        let locals = Locals { exp: 1. };
        encoder.update_constant_buffer(&self.bundle.data.locals, &locals);
        encoder.clear(&self.bundle.data.out, [0.0; 4]);
        self.bundle.encode(encoder);
    }


    fn on_resize(&mut self, window_targets: gfx_app::WindowTargets<R>) {
        self.bundle.data.out = window_targets.color;
    }
}

pub fn main() {
    use gfx_app::Application;
    let wb = winit::WindowBuilder::new().with_title("Canvas View");
    App::launch_default(wb);
}

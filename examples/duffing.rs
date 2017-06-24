#![feature(non_ascii_idents)]
#![feature(core_intrinsics)]
#![feature(conservative_impl_trait)]
#![feature(inclusive_range_syntax)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate gfx;
extern crate gfx_app;
extern crate gfx_core;
extern crate winit;

extern crate canvas;
extern crate tuple;
extern crate math;
extern crate futures;
extern crate cpal;
extern crate rand;
extern crate rusttype;
//extern crate fmath;

use gfx_app::{ColorFormat};
use canvas::plot::{LineStyle};
use tuple::{T2, TupleElements, Splat};
use canvas::array::{Array, RowMajor};
use canvas::canvas::{Canvas, Data, Meta};
use math::integrate::Integration;
use math::real::Real;
use math::cast::Cast;
//use fmath::*;
use gfx::handle::Texture;
use gfx::Bundle;
use gfx::texture;
use gfx_core::Resources;
use gfx_core::texture::NewImageInfo;
use gfx::format::{D32, R32, Float, Uint};
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use rusttype::{Font, FontCollection, Scale, Point};

lazy_static!{
    static ref LABEL_FONT: Font<'static> = FontCollection::from_bytes(
        &include_bytes!("fonts/LiberationSerif-Regular.ttf")[..]
    ).font_at(0).unwrap();
}

/*
fn cos(x: f32) -> f32 {
    let x = f32x8::splat(x);
    poly8_f32x8_avx(POLY32_COS_8_PI, x * x).extract(0)
}*/


#[derive(Copy, Clone, Debug)]
struct DuffingParams {
    epsilon:    f32,
    lambda:     f32,
    omega:      f32,
    alpha:      f32,
    beta:       f32
}

#[allow(non_snake_case)]
#[inline]
fn duffing(p: DuffingParams)
 -> impl Fn(f32, T2<f32, f32>) -> T2<f32, f32>
{
    // epsilon * cos(omega t) - lambda dx/dt - x * (alpha + x^2 * beta)
    use std::intrinsics::{fmul_fast, cosf32};
    move |t, s| {
        unsafe {
            T2(
                s.1,
                fmul_fast(p.epsilon, cosf32(fmul_fast(p.omega, t)))
                - fmul_fast(p.lambda, s.1)
                - fmul_fast(s.0, p.alpha + fmul_fast(fmul_fast(s.0, s.0), p.beta))
            )
        }
    }
}

// DuffingParams { epsilon: 17.780767, lambda: 0.6265935, omega: 4.5825043, alpha: 9.212474, beta: 0.005351869 }

impl Default for DuffingParams {
    fn default() -> DuffingParams {
        DuffingParams {
            epsilon: 7.72,
            lambda: 0.02,
            omega: 1.,
            alpha: 0.01,
            beta: 1.0
        }
    }
}

enum Command {
    Update(DuffingParams),
    Reset(DuffingParams, T2<f32, f32>, f32)
}

fn audio(rx: Receiver<Command>) {
    use std::sync::Arc;
    use futures::stream::Stream;
    use futures::task;
    use futures::task::Executor;
    use futures::task::Run;
    struct MyExecutor;

    impl Executor for MyExecutor {
        fn execute(&self, r: Run) {
            r.run();
        }
    }
    let endpoint = cpal::get_default_endpoint().expect("Failed to get default endpoint");
    let format = endpoint.get_supported_formats_list().unwrap()
    .find(|fmt| fmt.samples_rate.0 == 48000 && fmt.channels.len() == 2)
    .expect("Failed to get endpoint format");

    let event_loop = cpal::EventLoop::new();
    let executor = Arc::new(MyExecutor);

    let (mut voice, stream) = cpal::Voice::new(&endpoint, &format, &event_loop).expect("Failed to create a voice");

    println!("format: {:?}", format);
    
    // Produce a sinusoid of maximum amplitude.
    let samples_rate = format.samples_rate.0 as f32;
    let mut integrator = Integration::new(
        duffing(DuffingParams::default()), // the function to integrate
        T2(1.0, 1.0), // initial value
        0.0, // inital time
        440. / samples_rate // step size
    );
    
    voice.play();
    task::spawn(stream.for_each(move |buffer| -> Result<_, ()> {
        if let Ok(cmd) = rx.try_recv() {
            match cmd {
                Command::Update(params) => {
                    let t = integrator.t;
                    let y = integrator.y;
                    
                    integrator = Integration::new(
                        duffing(params), // the function to integrate
                        y, // initial value
                        t, // inital time
                        440. / samples_rate // step size
                    );
                },
                Command::Reset(params, y, t) => {
                    integrator = Integration::new(
                        duffing(params), // the function to integrate
                        y, // initial value
                        t, // inital time
                        440. / samples_rate // step size
                    );
                }
            }
        }
        let mut data_source = integrator.by_ref()
        .map(|v| v * T2(0.1, 0.02));
        
        match buffer {
            cpal::UnknownTypeBuffer::U16(mut buffer) => {
                for (sample, value) in buffer.chunks_mut(format.channels.len()).zip(&mut data_source) {
                    let value: T2<u16, u16> = value.map(|f| 
                        (0.5 * f + 0.5) * (std::u16::MAX as f32)
                    ).cast_clamped(
                        T2::splat(std::u16::MIN) ... T2::splat(std::u16::MAX)
                    );
                    
                    for (ch, out) in sample.iter_mut().enumerate() {
                        *out = value.get(ch).cloned().unwrap_or(0);
                    }
                }
            },

            cpal::UnknownTypeBuffer::I16(mut buffer) => {
                for (sample, value) in buffer.chunks_mut(format.channels.len()).zip(&mut data_source) {
                    let value: T2<i16, i16> = value.map(|f|
                        f * (std::i16::MAX as f32)
                    ).cast_clamped(
                        T2::splat(std::i16::MIN) ... T2::splat(std::i16::MAX)
                    );
                    
                    for (ch, out) in sample.iter_mut().enumerate() {
                        *out = value.get(ch).cloned().unwrap_or(0);
                    }
                }
            },

            cpal::UnknownTypeBuffer::F32(mut buffer) => {
                for (sample, value) in buffer.chunks_mut(format.channels.len()).zip(&mut data_source) {
                    for (ch, out) in sample.iter_mut().enumerate() {
                        *out = value.get(ch).cloned().unwrap_or(0.0);
                    }
                }
            },
        };

        Ok(())
    })).execute(executor);

    event_loop.run();
}


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
        labels: gfx::TextureSampler<f32> = "t_Labels",
        exp: gfx::Global<f32> = "i_Exp",
        size:   gfx::Global<[f32; 2]> = "i_Size",
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

enum Selector {
    Epsilon,
    Lambda,
    Omega,
    Alpha,
    Beta,
    Exp
}

#[derive(Debug, Copy, Clone)]
struct Params {
    duffing:    DuffingParams,
    exp:        f32
}
    

const LABEL_HEIGHT: usize = 30;
const NUM_LABELS: usize = 5;
struct App<R: Resources>{
    bundle:     Bundle<R, pipe::Data<R>>,
    integrator: (T2<f32, f32>, f32),
    map:        Array<Vec<u32>, RowMajor>,
    tex_canvas: Texture<R, R32>,
    tex_labels: Texture<R, D32>,
    params:     Params,
    select:     Selector,
    tx:         Sender<Command>,
    label_queue: Vec<(Vec<f32>, NewImageInfo)>,
}

impl<R: Resources> App<R> {
    fn trace(&mut self, iterations: usize) {
        let ref mut rng = rand::thread_rng();
        let size: T2<usize, usize> = self.map.meta.size().into();
        let sizef: T2<f32, f32> = size.cast().unwrap();
        let bounds = T2(20.0f32, 100.0f32);
        
        let canvas_scale = sizef / bounds;
        let offset = T2(10., 50.);
        let one = (1u8).cast().unwrap();
        
        let (y0, t0) = self.integrator;
        
        let mut integration = Integration::new(
            duffing(self.params.duffing),
            y0,
            t0,
            1e-3
        );
        self.map.run_mut(|meta, data| 
            data.apply(
                integration.by_ref()
                .take(iterations)
                .map(|p| (p + offset) * canvas_scale)
                .map(|p| p + T2::uniform01(rng))
                .filter_map(|p: T2<f32, f32>| p.cast_clipped(T2(0, 0) ... size - T2(1, 1)))
                .map(|T2(x, y)| (meta.index((x, y)), one)),
                
                |v, increment| v + increment
            )
        );
        
        self.integrator = (integration.y, integration.t);
    }
    
    pub fn update_label(&mut self, label: usize) {
        let p = &self.params.duffing;
        let text = match label {
            0 => format!("ɛ: {:10.7}", p.epsilon),
            1 => format!("λ: {:10.7}", p.lambda),
            2 => format!("Ω: {:10.7}", p.omega),
            3 => format!("α: {:10.7}", p.alpha),
            4 => format!("β: {:10.7}", p.beta),
            _ => panic!()
        };
    
        let size: T2<usize, usize> = self.map.meta.size().into();
        let width = size.0 / NUM_LABELS;
        let mut buffer = vec![0.0f32; width * LABEL_HEIGHT]; // TODO: avoid allocation (jenga?)
        let scale = Scale::uniform(20.);
        let start = Point { x: 10., y: LABEL_HEIGHT as f32 - 20. * 0.5 };
        for glyph in LABEL_FONT.layout(&text, scale, start) {
            if let Some(bb) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    let x = x as i32 + bb.min.x;
                    let y = y as i32 + bb.min.y;
                // println!("{} {} {}", x, y, v);
                    if x >= 0 && (x as usize) < width && y >= 0 && (y as usize) < LABEL_HEIGHT {
                        buffer[x as usize + (y as usize) * width] += v;
                    }
                });
            }
        }
        let info = NewImageInfo {
            xoffset:    (label * width) as u16,
            yoffset:    0,
            zoffset:    0,
            width:      width as u16,
            height:     LABEL_HEIGHT as u16,
            depth:      0,
            format:     (),
            mipmap:     0
        };
        self.label_queue.push((buffer, info));
    }

}

impl<R: Resources> gfx_app::Application<R> for App<R> {
    fn new<F: gfx::Factory<R>>(factory: &mut F, backend: gfx_app::shade::Backend,
           window_targets: gfx_app::WindowTargets<R>) -> Self
    {
        use gfx::traits::FactoryExt;
        use gfx_core::shade::CreateShaderError::CompilationFailed;
        use gfx::shade::ProgramError::Pixel;
        use gfx::PipelineStateError::Program;
        use gfx::texture::{SamplerInfo, WrapMode, FilterMethod};
        
        let width = 1366;
        let height = 768;
        let map = Array::new(RowMajor::new(width, height), vec![0; width * height]);
        
        let vs = gfx_app::shade::Source {
            glsl_150: include_bytes!("shader/canvas_150.glslv"),
            .. gfx_app::shade::Source::empty()
        };
        let ps = gfx_app::shade::Source {
            glsl_150: include_bytes!("shader/canvas_150.glslf"),
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
        let kind_canvas = texture::Kind::D2(
            width as texture::Size,
            height as texture::Size,
            texture::AaMode::Single
        );
        let kind_labels = texture::Kind::D2(
            width as texture::Size,
            LABEL_HEIGHT as texture::Size,
            texture::AaMode::Single
        );
        let tex_canvas = factory.create_texture::<R32>(
            kind_canvas, 1,
            gfx_core::memory::SHADER_RESOURCE,
            gfx::memory::Usage::Dynamic, Some(gfx::format::ChannelType::Uint)
        ).expect("create canvas texture");
        let tex_labels = factory.create_texture::<D32>(
            kind_labels, 1,
            gfx_core::memory::SHADER_RESOURCE,
            gfx::memory::Usage::Dynamic, Some(gfx::format::ChannelType::Float)
        ).expect("create labels texture");
        
        let pso = factory.create_pipeline_simple(
            vs.select(backend).expect("failed vertex shader"),
            ps.select(backend).expect("failed pixel shader"),
            pipe::new()
        ).unwrap();
        
        
        let cbuf = factory.create_constant_buffer(1);
        let canvas = factory.view_texture_as_shader_resource::<(R32, Uint)>(&tex_canvas, (0, 0), gfx::format::Swizzle::new()).expect("as shader ressource");
        
        let labels = factory.view_texture_as_shader_resource::<(D32, Float)>(&tex_labels, (0, 0), gfx::format::Swizzle::new()).expect("as shader ressource");
        
        let data = pipe::Data {
            vbuf:   vbuf,
            canvas: (canvas, factory.create_sampler(SamplerInfo::new(FilterMethod::Scale, WrapMode::Clamp))),
            labels: (labels, factory.create_sampler_linear()),
            exp:    1.,
            size:   [width as f32, height as f32],
            locals: cbuf,
            out:    window_targets.color.clone(),
        };
        
        let (tx, rx) = channel();
        thread::spawn(move || audio(rx));
        
        let mut app = App {
            bundle:     Bundle::new(slice, pso, data),
            map:        map,
            integrator: (T2(0.5, 0.0), 0.0),
            tex_canvas: tex_canvas,
            tex_labels: tex_labels,
            params:     Params {
                duffing:    DuffingParams::default(),
                exp:        1.0
            },
            select:     Selector::Epsilon,
            tx:         tx,
            label_queue: vec![]
        };
        
        for p in 0 .. 5 {
            app.update_label(p);
        }
        
        app
    }

    
    fn render<C: gfx::CommandBuffer<R>>(&mut self, encoder: &mut gfx::Encoder<R, C>) {
        self.trace(50_000);
        for (buffer, info) in self.label_queue.drain(..) {
            encoder.update_texture::<gfx::format::D32, (D32, Float)>(
                &self.tex_labels, None, info, &buffer
            ).expect("update texture");
        }
        let view_info = self.tex_canvas.get_info().to_image_info(0);
        encoder.update_texture::<gfx::format::R32, (R32, Uint)>(&self.tex_canvas, None, view_info, &self.map.data).expect("update texture");
        self.bundle.data.exp = self.params.exp;
        let locals = Locals { exp: self.params.exp };
        encoder.update_constant_buffer(&self.bundle.data.locals, &locals);
        encoder.clear(&self.bundle.data.out, [0.0; 4]);
        self.bundle.encode(encoder);
    }
    
    fn on(&mut self, event: winit::WindowEvent) {
        use winit::{VirtualKeyCode, MouseScrollDelta, WindowEvent};
        use self::Selector::*;
        
        match event {
            WindowEvent::MouseWheel(delta, _) => {
                let mul = match delta {
                    MouseScrollDelta::LineDelta(dx, dy) => 1.05f32.powf(dy),
                    MouseScrollDelta::PixelDelta(dx, dy) => 1.01f32.powf(dy)
                };
                let mut p = self.params;
                let label = match self.select {
                    Epsilon =>  { p.duffing.epsilon *= mul; Some(0) },
                    Lambda =>   { p.duffing.lambda *= mul;  Some(1) },
                    Omega =>    { p.duffing.omega *= mul;   Some(2) },
                    Alpha =>    { p.duffing.alpha *= mul;   Some(3) },
                    Beta =>     { p.duffing.beta *= mul;    Some(4) }
                    Exp =>      { p.exp *= mul;             None    }
                };
                self.params = p;
                if let Some(label) = label {
                    self.tx.send(Command::Update(p.duffing)).unwrap();
                    println!("{:?}", p);
                    self.update_label(label);
                }
            },
            WindowEvent::KeyboardInput(_, _, Some(key), _) => match key {
                VirtualKeyCode::Key1 => self.select = Epsilon,
                VirtualKeyCode::Key2 => self.select = Lambda,
                VirtualKeyCode::Key3 => self.select = Omega,
                VirtualKeyCode::Key4 => self.select = Alpha,
                VirtualKeyCode::Key5 => self.select = Beta,
                VirtualKeyCode::Key0 => self.select = Exp,
                VirtualKeyCode::C => {
                    for v in self.map.data.iter_mut() {
                        *v = 0;
                    }
                    let (y, t) = (T2(1.0, 0.0), 0.0);
                    self.integrator = (y, t);
                    self.tx.send(Command::Reset(self.params.duffing, y, t));
                },
                _ => ()
            },
            _ => ()
        }
    }

    fn on_resize(&mut self, window_targets: gfx_app::WindowTargets<R>) {
        self.bundle.data.out = window_targets.color;
    }
}

pub fn main() {
    use gfx_app::Application;
    let wb = winit::WindowBuilder::new()
    .with_title("Canvas View");
    App::launch_default(wb);
}

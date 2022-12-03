use std::num::NonZeroU32;
use clap::{ Parser };
use ndarray::{ Array1, Array2 };
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle,
    RawWindowHandle,
};
use winit::{
    event::{ Event, WindowEvent },
    event_loop::{ ControlFlow, EventLoop },
    window::{ Window, WindowBuilder },
};
#[cfg(glx_backend)]
use winit::platform::unix;

use glutin::{
    config::{ Config, ConfigSurfaceTypes, ConfigTemplateBuilder },
    context::{ ContextApi, ContextAttributesBuilder, NotCurrentContext },
    display::{ Display, DisplayApiPreference },
    prelude::*,
    surface::{ Surface, SurfaceAttributesBuilder, WindowSurface },
};

pub mod atom;
pub mod basis;
pub mod controller;
pub mod drawer;
pub mod error;
pub mod event;
pub mod gl;
pub mod mo_coefs;

use atom::Atom;
use basis::Basis;
use drawer::Drawer;
use controller::Controller;
use event::{ Frame, HandleEvent };

#[derive(Parser, Debug)]
#[command(name = "CH121 Final MO Drawer")]
#[command(author = "James Moore <jam0152@uah.edu>")]
#[command(about = "Visualizes molecular electron orbitals", long_about = None)]
struct Args {
    #[arg(short = 'B', long = "basis")]
    bases: Vec<String>,
    #[arg(short = 'A', long = "atom")]
    atoms: Vec<String>,
    #[arg(short = 'C', long = "coefs")]
    coefs: String,
}

fn collect_argvs(raw_argv: &Vec<String>) -> Vec<Vec<String>> {
    let mut argvs = Vec::new();

    for arg in raw_argv {
        if arg.starts_with("[") {
            let arg = String::from(arg.trim_start_matches("["));

            argvs.push(vec![arg]);
        } else {
            let arg = String::from(arg.trim_end_matches("]"));

            argvs.last_mut()
                 .unwrap()
                 .push(arg);
        }
    }

    argvs
}

#[derive(Debug)]
pub struct Params {
    pub bases:      Array1<Basis>,
    pub atoms:      Array1<Atom>,
    pub mo_coefs:   Array2<f64>,
}

impl Params {
    fn from_args(args: Args) -> Self {
        let bases = args.bases.into_iter()
                              .map(|s| Basis::from_arg(&s).unwrap())
                              .collect();

        let atoms = args.atoms.into_iter()
                              .map(|s| Atom::from_arg(&s).unwrap())
                              .collect();

        let mo_coefs = mo_coefs::from_arg(&args.coefs)
                                .unwrap();

        Self { bases, atoms, mo_coefs }
    }
}

#[allow(unused_variables)]
fn create_gl_display(raw_display: RawDisplayHandle,
                     raw_window: RawWindowHandle) -> Display {
    #[cfg(egl_backend)]
    let preference = DisplayApiPreference::Egl;

    #[cfg(glx_backend)]
    let preference = DisplayApiPreference::Glx(Box::new(unix::register_xlib_error_hook));

    #[cfg(cgl_backend)]
    let preference = DisplayApiPreference::Cgl;

    #[cfg(wgl_backend)]
    let preference = DisplayApiPreference::Wgl(Some(raw_window));

    #[cfg(all(egl_backend, wgl_backend))]
    let preference = DisplayApiPreference::WglThenEgl(Some(raw_window));

    #[cfg(all(egl_backend, glx_backend))]
    let preference = DisplayApiPreference::GlxThenEgl(Box::new(unix::register_xlib_error_hook));

    unsafe { Display::new(raw_display, preference)
                     .unwrap() }
}

fn find_gl_config(gl_display: &Display,
                  raw_window: RawWindowHandle) -> Config {
    let template = {
        let builder = ConfigTemplateBuilder::new()
                                            .with_alpha_size(8)
                                            .compatible_with_native_window(raw_window)
                                            .with_surface_type(ConfigSurfaceTypes::WINDOW);
        #[cfg(cgl_backend)]
        let builder = builder.with_multisampling(8);

        builder.build()
    };

    unsafe {
        gl_display.find_configs(template)
                  .unwrap()
                  .reduce(|acc, c| {
                      if c.num_samples() > acc.num_samples() {
                          c
                      } else {
                          acc
                      }
                  })
                  .unwrap()
    }
}

fn create_gl_context(gl_display: &Display,
                     raw_window: RawWindowHandle,
                     gl_config: &Config) -> NotCurrentContext {
    let ctx_attrs =
        ContextAttributesBuilder::new()
                                 .build(Some(raw_window));
    let fallback_ctx_attrs =
        ContextAttributesBuilder::new()
                                 .with_context_api(ContextApi::Gles(None))
                                 .build(Some(raw_window));
    unsafe {
        gl_display.create_context(&gl_config, &ctx_attrs)
                  .unwrap_or_else(|_| {
        gl_display.create_context(&gl_config, &fallback_ctx_attrs)
                  .expect("failed to create gl context")
        })
    }
}

fn create_gl_surface(gl_display:    &Display,
                     window:        &Window,
                     gl_config:     &Config) -> Surface<WindowSurface> {
    let size = window.inner_size();
    let width = NonZeroU32::new(size.width)
                           .unwrap();
    let height = NonZeroU32::new(size.height)
                            .unwrap();
    let raw_window = window.raw_window_handle();
    let attrs =
        SurfaceAttributesBuilder::<WindowSurface>
                                ::new()
                                 .build(raw_window, width, height);

    unsafe { gl_display.create_window_surface(gl_config, &attrs)
                       .unwrap() }
}

fn main() {
    let args = Args::parse();
    let params = Params::from_args(args);

    let event_loop = EventLoop::new();
    let raw_display = event_loop.raw_display_handle();
    let window = WindowBuilder::new()
                               .with_title("mo-draw")
                               .build(&event_loop)
                               .unwrap();
    let raw_window = window.raw_window_handle();
    let gl_display = create_gl_display(raw_display, raw_window);
    let gl_config = find_gl_config(&gl_display, raw_window);
    let gl_surface = create_gl_surface(&gl_display, &window, &gl_config);
    let gl_context = create_gl_context(&gl_display, raw_window, &gl_config);
    let gl_context = gl_context.make_current(&gl_surface)
                               .unwrap();

    let mut frame = Frame::initial();
    let mut controller = Controller::new();
    let drawer = Drawer::new(&gl_display, &params);

    event_loop.run(move |evt, _, ctl_flow| {
        *ctl_flow = ControlFlow::Wait;

        frame.next();
        controller.handle_event(&frame, &evt);

        match evt {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    if size.width != 0 && size.height != 0 {
                        let width = NonZeroU32::new(size.width)
                                               .unwrap();
                        let height = NonZeroU32::new(size.height)
                                                .unwrap();

                        gl_surface.resize(&gl_context, width, height);
                        drawer.resize(size.width, size.height);
                    }
                },
                WindowEvent::CloseRequested => {
                    *ctl_flow = ControlFlow::Exit;
                },
                _ => (),
            },
            Event::RedrawEventsCleared => {
                drawer.draw_mo(controller.vars.mo_idx);
                window.request_redraw();

                gl_surface.swap_buffers(&gl_context)
                          .unwrap();
            },
            _ => (),
        }
    });
}

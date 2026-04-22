
pub mod renderer;
pub mod camera;

#[cfg(debug_assertions)]
extern "system" fn gl_debug_output(
	source: gl::types::GLenum,
    gl_type: gl::types::GLenum,
    id: gl::types::GLuint,
    severity: gl::types::GLenum,
    _length: gl::types::GLsizei,
    message: *const gl::types::GLchar,
    _user_param: *mut std::ffi::c_void,
) {
	let source_str = match source {
		gl::DEBUG_SOURCE_API             => "API",
		gl::DEBUG_SOURCE_WINDOW_SYSTEM   => "Window System",
		gl::DEBUG_SOURCE_SHADER_COMPILER => "Shader Compiler",
		gl::DEBUG_SOURCE_THIRD_PARTY     => "Third Party",
		gl::DEBUG_SOURCE_APPLICATION     => "Application",
		gl::DEBUG_SOURCE_OTHER           => "Other",
		_                                => "Unknown",
	};

	let type_str = match gl_type {
		gl::DEBUG_TYPE_ERROR               => "Error",
		gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecated",
		gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR  => "Undefined Behavior",
		gl::DEBUG_TYPE_PORTABILITY         => "Portability",
		gl::DEBUG_TYPE_PERFORMANCE         => "Performance",
		gl::DEBUG_TYPE_MARKER              => "Marker",
		gl::DEBUG_TYPE_PUSH_GROUP          => "Push Group",
		gl::DEBUG_TYPE_POP_GROUP           => "Pop Group",
		gl::DEBUG_TYPE_OTHER               => "Other",
		_                                  => "Unknown",
	};

	let severity_str = match severity {
		gl::DEBUG_SEVERITY_HIGH         => "\x1b[31mHigh\x1b[0m",   // Vermelho
		gl::DEBUG_SEVERITY_MEDIUM       => "\x1b[33mMedium\x1b[0m", // Amarelo
		gl::DEBUG_SEVERITY_LOW          => "\x1b[32mLow\x1b[0m",    // Verde
		gl::DEBUG_SEVERITY_NOTIFICATION => "\x1b[34mNote\x1b[0m",   // Azul
		_                               => "Unknown",
	};

	let message_str = unsafe {
        use std::ffi::CStr;
		CStr::from_ptr(message).to_str().unwrap_or("Failed to parse message")
	};

	eprintln!(
		"[GL Debug] [{}] Type: {} | Source: {} | ID: {}\nMessage: {}\n",
		severity_str, type_str, source_str, id, message_str
	);
}

use std::num::{NonZero, NonZeroU32};
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, GlProfile, PossiblyCurrentContext, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::HasWindowHandle;
use winit::window::{Window, WindowId};
use crate::graphics::renderer::Renderer;
use nalgebra_glm as glm;

const RENDERER: Renderer = Renderer::new();

pub trait ApplicationState<'a> {
    fn get_window_title() -> &'static str;
    fn get_window_size() -> glm::Vec2;
    fn new(window: Window, gl_context: PossiblyCurrentContext, gl_surface: Surface<WindowSurface>, renderer: &'a Renderer) -> Self;
    fn event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent);
}

pub struct App<'a, T: ApplicationState<'a>> {
    state: Option<T>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a, T: ApplicationState<'a>> App<'a, T> {
    pub fn new() -> Self {
        Self { state: None, _marker: std::marker::PhantomData }
    }
}

impl<'a, T: ApplicationState<'a>> ApplicationHandler for App<'a, T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_size = T::get_window_size();
        let window_attrs = Window::default_attributes()
            .with_title(T::get_window_title())
            .with_inner_size(winit::dpi::LogicalSize::new(window_size.x, window_size.y));

        let template = ConfigTemplateBuilder::new()
            .with_multisampling(4)
            .with_alpha_size(0); 

        let display_builder = glutin_winit::DisplayBuilder::new()
            .with_window_attributes(Some(window_attrs));

        let (window, gl_config) = display_builder
            .build(event_loop, template, |mut configs| {
                configs.next().unwrap()
            })
            .unwrap();

        let window = window.unwrap();
        let handle = window.window_handle().unwrap().as_raw();

        let mut context_attrs_builder = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(4, 6))))
            .with_profile(GlProfile::Core);

        if cfg!(debug_assertions) {
            context_attrs_builder = context_attrs_builder.with_debug(true);
        }

        let context_attrs = context_attrs_builder.build(Some(handle));

        let gl_context = unsafe {
            gl_config.display().create_context(&gl_config, &context_attrs).expect("Falha ao criar contexto GL")
        };

        let size = window.inner_size();
        let surface_attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            handle,
            NonZeroU32::new(size.width).unwrap(),
            NonZeroU32::new(size.height).unwrap(),
        );

        let gl_surface = unsafe {
            gl_config.display().create_window_surface(&gl_config, &surface_attrs).unwrap()
        };

        let gl_context = gl_context.make_current(&gl_surface).unwrap();

        gl_surface.set_swap_interval(&gl_context, SwapInterval::Wait(NonZero::new(1).unwrap())).unwrap();

        gl::load_with(|symbol| {
            let symbol = std::ffi::CString::new(symbol).unwrap();
            gl_config.display().get_proc_address(&symbol) as *const _
        });

        #[cfg(debug_assertions)]
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(gl_debug_output), std::ptr::null());
        }

        self.state = Some(T::new(window, gl_context, gl_surface, &RENDERER));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(state) = &mut self.state {
            state.event(event_loop, event);
        }
    }
}

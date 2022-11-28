use std::{
    ffi::{ CStr, CString },
    marker::PhantomData, mem, ptr,
};
use crate::{
    error::Error,
    gl::{ self, types::* },
    Params
};
use glutin::display::{ Display, GlDisplay };
use nalgebra::{ Rotation3, Translation3 };

mod c_fmt;

macro_rules! c_str {
    ($s:literal) => {
        concat!($s, '\0').as_ptr() as *const i8
    }
}

struct Shader {
    gl_handle:  GLuint,
}

impl Shader {
    fn new(ty: GLenum, src: &str) -> Result<Self, Error> {
        let src = CString::new(src)?;

        unsafe {
            let gl_handle = gl::CreateShader(ty);

            gl::ShaderSource(gl_handle, 1,
                             [src.as_ptr()].as_ptr(),
                             ptr::null());
            gl::CompileShader(gl_handle);

            let mut is_compiled = mem::zeroed();
            gl::GetShaderiv(gl_handle, gl::COMPILE_STATUS, &mut is_compiled);

            if is_compiled == gl::FALSE as i32 {
                let mut log_len = mem::zeroed();
                gl::GetShaderiv(gl_handle, gl::INFO_LOG_LENGTH, &mut log_len);

                let mut log_buf = Vec::new();
                log_buf.resize(log_len as usize, 0u8);
                gl::GetShaderInfoLog(gl_handle,
                                     log_len,
                                     &mut log_len,
                                     log_buf.as_mut_ptr() as *mut i8);

                let log = String::from_utf8(log_buf)
                                 .unwrap();

                println!("{}", log);

                Err(log.into())
            } else {
                Ok(Self { gl_handle })
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.gl_handle); }
    }
}

struct Program {
    gl_handle:  GLuint,
    resolution: GLint,
    mo_idx:     GLint,
}

impl Program {
    fn new(params: &Params) -> Result<Self, Error> {
        unsafe {
            const VERT_SRC: &'static str = include_str!("vert.glsl");
            let vert = Shader::new(gl::VERTEX_SHADER, VERT_SRC)?;

            const FRAG_TEMPLATE: &'static str = include_str!("frag_template.glsl");
            let num_cc = params.bases[0].order.to_string();
            let orbitals = c_fmt::orbitals(&params.atoms, &params.bases);
            let mo_coefs = c_fmt::array2(&params.mo_coefs);
            let frag_src = String::from(FRAG_TEMPLATE)
                                  .replace("@STEP_LEN", "0.1")
                                  .replace("@NUM_STEPS", "200")
                                  .replace("@NUM_CC", &num_cc)
                                  .replace("@ORBITALS", &orbitals)
                                  .replace("@MO_COEFS", &mo_coefs);
            let frag = Shader::new(gl::FRAGMENT_SHADER, frag_src.as_str())?;

            let gl_handle = gl::CreateProgram();
            gl::AttachShader(gl_handle, vert.gl_handle);
            gl::AttachShader(gl_handle, frag.gl_handle);
            gl::LinkProgram(gl_handle);

            let resolution = gl::GetUniformLocation(gl_handle, c_str!("resolution"));
            let mo_idx = gl::GetUniformLocation(gl_handle, c_str!("mo_idx"));

            Ok(Self { gl_handle, resolution, mo_idx })
        }
    }

    fn bind(&self) {
        unsafe { gl::UseProgram(self.gl_handle); }
    }

    fn set_resolution(&self, width: f32, height: f32) {
        unsafe { gl::Uniform2f(self.resolution, width, height); }
    }

    fn set_mo_idx(&self, mo_idx: i32) {
        unsafe { gl::Uniform1i(self.mo_idx, mo_idx); }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.gl_handle); }
    }
}

struct VertexLayout<T> {
    gl_handle:  GLuint,
    phantom:    PhantomData<T>,
}

impl<T> VertexLayout<T> {
    fn builder() -> VertexLayoutBuilder<T> {
        VertexLayoutBuilder::new()
    }

    fn from_raw_handle(gl_handle: GLuint) -> Self {
        Self { gl_handle, phantom: PhantomData }
    }

    fn bind(&self) {
        unsafe { gl::BindVertexArray(self.gl_handle); }
    }
}

impl<T> Drop for VertexLayout<T> {
    fn drop(&mut self) {
        unsafe { gl::DeleteVertexArrays(1, &self.gl_handle); }
    }
}

struct VertexLayoutBuilder<T> {
    attrs:      Vec<(GLuint, GLint, GLenum, GLboolean, usize)>,
    phantom:    PhantomData<T>
}

impl<T> VertexLayoutBuilder<T> {
    fn new() -> Self {
        Self { attrs: Vec::new(), phantom: PhantomData }
    }

    fn attr(mut self,
            idx:    usize,
            arity:  usize,
            ty:     GLenum,
            norm:   bool) -> Self {
        let size = arity * match ty {
            gl::FLOAT => mem::size_of::<f32>(),
            _ => unimplemented!(),
        };

        self.attrs.push((idx as GLuint, arity as GLint, ty, norm as GLboolean, size));
        self
    }

    fn build(self) -> VertexLayout<T> {
        unsafe {
            let mut gl_handle = mem::zeroed();
            gl::GenVertexArrays(1, &mut gl_handle);
            gl::BindVertexArray(gl_handle);

            let mut offset: usize = 0;

            for (idx, arity, ty, norm, size) in self.attrs {
                let end = offset + size;
                assert!(end <= mem::size_of::<T>());

                gl::VertexAttribFormat(idx, arity, ty, norm,
                                       offset as GLuint);
                gl::VertexAttribBinding(idx, 0);
                gl::EnableVertexAttribArray(idx);

                offset = end;
            }

            VertexLayout::from_raw_handle(gl_handle)
        }
    }
}

struct VertexBuffer<T> {
    gl_handle:  GLuint,
    len:        usize,
    layout:     VertexLayout<T>,
}

impl<T> VertexBuffer<T> {
    fn from_slice(slice: &[T], layout: VertexLayout<T>) -> Self {
        let len = slice.len();
        let size = len * mem::size_of::<T>();
        let data = slice.as_ptr();

        unsafe {
            let mut gl_handle = mem::zeroed();

            gl::GenBuffers(1, &mut gl_handle);
            gl::BindBuffer(gl::ARRAY_BUFFER, gl_handle);
            gl::BufferData(gl::ARRAY_BUFFER,
                           size as GLsizeiptr,
                           data as *const _,
                           gl::STATIC_DRAW);

            Self { gl_handle, len, layout }
        }
    }

    fn draw(&self, mode: GLenum) {
        self.layout.bind();

        unsafe {
            let stride = mem::size_of::<T>();
            gl::BindVertexBuffer(0, self.gl_handle, 0, stride as GLsizei);
            gl::DrawArrays(mode, 0, self.len as GLsizei);
        }
    }
}

impl<T> Drop for VertexBuffer<T> {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &self.gl_handle); }
    }
}

unsafe fn load_opengl(gl_display: &Display) {
    gl::load_with(|sym| {
        let sym = CString::new(sym)
                          .unwrap();

        gl_display.get_proc_address(sym.as_c_str())
                  .cast()
    });
}

fn get_gl_env(iden: GLenum) -> Option<&'static str> {
    unsafe {
        let s = gl::GetString(iden);

        (!s.is_null()).then(|| {
            CStr::from_ptr(s.cast())
                 .to_str()
                 .unwrap()
        })
    }
}

pub struct Drawer {
    program:    Program,
    buffer:     VertexBuffer<[f32; 2]>,
}

impl Drawer {
    pub fn new(gl_display: &Display, params: &Params) -> Self {
        #[rustfmt::skip]
        static VERTEX_DATA: [[f32; 2]; 4] = [
            [ -1.0, -1.0, ],
            [  1.0, -1.0, ],
            [ -1.0,  1.0, ],
            [  1.0,  1.0, ],
        ];

        unsafe {
            load_opengl(gl_display);

            if let Some(renderer) = get_gl_env(gl::RENDERER) {
                println!("Running on {}", renderer);
            }

            if let Some(version) = get_gl_env(gl::VERSION) {
                println!("OpenGL version {}", version);
            }

            if let Some(glsl_version) = get_gl_env(gl::SHADING_LANGUAGE_VERSION) {
                println!("GLSL version {}", glsl_version);
            }

            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(Some(gl_debug_callback),
                                     ptr::null());
        }

        let program = Program::new(params)
                              .unwrap();
        let layout = VertexLayout::builder()
                                  .attr(0, 2, gl::FLOAT, false)
                                  .build();
        let buffer = VertexBuffer::from_slice(&VERTEX_DATA, layout);

        /*unsafe {
            gl::VertexAttribPointer(0, 2,
                                    gl::FLOAT,
                                    gl::FALSE,
                                    (2 * mem::size_of::<f32>()) as GLsizei,
                                    ptr::null());
            gl::EnableVertexAttribArray(0);
        }*/

        Self { program, buffer }
    }

    pub fn draw_mo(&self, mo_idx: usize) {
        self.program.bind();
        self.program.set_mo_idx(mo_idx as i32);

        unsafe {
            gl::ClearColor(0.9, 0.9, 0.9, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        self.buffer.draw(gl::TRIANGLE_STRIP);
    }

    pub fn resize(&self, width: u32, height: u32) {
        let (width, height) = (width as i32, height as i32);
        unsafe { gl::Viewport(0, 0, width, height); }

        self.program.set_resolution(width as f32, height as f32);
    }
}

extern "system" fn gl_debug_callback(source:        GLenum,
                                     ty:            GLenum,
                                     id:            GLuint,
                                     severity:      GLenum,
                                     length:        GLsizei,
                                     message:       *const GLchar,
                                     _user_param:   *mut GLvoid) {
    let source = match source {
        gl::DEBUG_SOURCE_API => "opengl",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "window system",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "shader compiler",
        gl::DEBUG_SOURCE_THIRD_PARTY => "third party",
        gl::DEBUG_SOURCE_APPLICATION => "application",
        gl::DEBUG_SOURCE_OTHER => "other",
        _ => panic!("invalid opengl debug message"),
    };

    let ty = match ty {
        gl::DEBUG_TYPE_ERROR => "error",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "deprecated behavior",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "undefined behavior",
        gl::DEBUG_TYPE_PORTABILITY => "portability",
        gl::DEBUG_TYPE_PERFORMANCE => "performance",
        gl::DEBUG_TYPE_MARKER => "marker",
        gl::DEBUG_TYPE_PUSH_GROUP => "group push",
        gl::DEBUG_TYPE_POP_GROUP => "group pop",
        gl::DEBUG_TYPE_OTHER => "other",
        _ => panic!("invalid opengl debug message"),
    };

    let severity = match severity {
        gl::DEBUG_SEVERITY_HIGH => "error",
        gl::DEBUG_SEVERITY_MEDIUM | gl::DEBUG_SEVERITY_LOW => "warning",
        gl::DEBUG_SEVERITY_NOTIFICATION => "info",
        _ => panic!("invalid opengl debug message"),
    };

    let message = unsafe {
        use std::slice;

        let slice = slice::from_raw_parts(message as *const u8,
                                          length as usize + 1);

        let s = CStr::from_bytes_with_nul(slice)
                     .unwrap()
                     .to_str()
                     .unwrap()
                     .trim_end();

        String::from(s)
    };

    println!("[{id:#018x}|{source:<16}|{severity:<8}|{ty:<20}] {message}");
}

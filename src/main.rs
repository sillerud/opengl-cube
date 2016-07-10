extern crate gl;
extern crate glutin;
extern crate libc;
extern crate cgmath;

use gl::types::*;
use std::io::prelude::*;
use std::mem;
use std::ptr;
use std::path::Path;
use std::str;
use std::fs::File;
use std::ffi::CString;

use cgmath::{ Matrix, Matrix4, One, PerspectiveFov, Point3, Vector3 };

struct Game {
    running: bool
}

static GL_VERTEX_DATA: [GLfloat; 108] = [
    -1.0, -1.0, -1.0, // Front
    1.0, -1.0, -1.0,
    -1.0, 1.0, -1.0,

    1.0, -1.0, -1.0,
    1.0, 1.0, -1.0,
    -1.0, 1.0, -1.0,

    -1.0, -1.0, 1.0, // Back
    1.0, -1.0, 1.0,
    -1.0, 1.0, 1.0,

    1.0, -1.0, 1.0,
    1.0, 1.0, 1.0,
    -1.0, 1.0, 1.0,

    1.0, -1.0, -1.0, // Right
    1.0, -1.0, 1.0,
    1.0, 1.0, -1.0,

    1.0, 1.0, 1.0,
    1.0, 1.0, -1.0,
    1.0, -1.0, 1.0,

    -1.0, -1.0, -1.0, // Left
    -1.0, -1.0, 1.0,
    -1.0, 1.0, -1.0,

    -1.0, 1.0, 1.0,
    -1.0, 1.0, -1.0,
    -1.0, -1.0, 1.0,

    -1.0, -1.0, -1.0, // Bottom
    1.0, -1.0, -1.0,
    -1.0, -1.0, 1.0,

    1.0, -1.0, 1.0,
    -1.0, -1.0, 1.0,
    1.0, -1.0, -1.0,

    -1.0, 1.0, -1.0, // Top
    1.0, 1.0, -1.0,
    -1.0, 1.0, 1.0,

    1.0, 1.0, 1.0,
    -1.0, 1.0, 1.0,
    1.0, 1.0, -1.0,


];

static GL_COLOR_DATA: [GLfloat; 108] = [
    1.0, 1.0, 1.0, // Front
    0.5, 0.5, 0.5,
    0.5, 0.5, 0.5,

    0.5, 0.5, 0.5,
    0.0, 0.0, 0.0,
    0.5, 0.5, 0.5,

    0.0, 1.0, 0.0, // Back
    0.0, 0.5, 0.0,
    0.0, 0.5, 0.0,

    0.0, 0.5, 0.0,
    0.0, 0.0, 0.0,
    0.0, 0.5, 0.0,

    1.0, 0.0, 0.0, //Right
    0.5, 0.0, 0.0,
    0.5, 0.0, 0.0,

    0.0, 0.0, 0.0,
    0.5, 0.0, 0.0,
    0.5, 0.0, 0.0,

    0.0, 0.0, 1.0, //Left
    0.0, 0.0, 0.5,
    0.0, 0.0, 0.5,

    0.0, 0.0, 0.0,
    0.0, 0.0, 0.5,
    0.0, 0.0, 0.5,

    0.0, 1.0, 1.0, //Bottom
    0.0, 0.5, 0.5,
    0.0, 0.5, 0.5,

    0.0, 0.0, 0.0,
    0.0, 0.5, 0.5,
    0.0, 0.5, 0.5,

    1.0, 0.0, 1.0, //Bottom
    0.5, 0.0, 0.5,
    0.5, 0.0, 0.5,

    0.0, 0.0, 0.0,
    0.5, 0.0, 0.5,
    0.5, 0.0, 0.5,
];

fn compile_shader<T: AsRef<Path>>(shader_path: T, typ: GLenum) -> GLuint {
    let mut shader_src = String::new();
    File::open(shader_path).unwrap().read_to_string(&mut shader_src).unwrap();
    unsafe {
        let shader = gl::CreateShader(typ);
        gl::ShaderSource(shader, 1, &CString::new(shader_src.as_bytes()).unwrap().as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("{}", str::from_utf8(&buf).ok().expect("ShaderInfoLog not valid utf8"));
        }
        shader
    }
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("{}", str::from_utf8(&buf).ok().expect("Error had invalid utf8"));
        }
        program
    }
}

fn main() {
    let window = glutin::Window::new().unwrap();
    let mut game = Game {
        running: true
    };

    unsafe {
        window.make_current();
    }
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let mut vao = 0;
    let mut vbo = 0;
    let mut color_buffer = 0;

    unsafe {
        // Enable depth test
        gl::Enable(gl::DEPTH_TEST);
        // Accept fragment if it closer to the camera than the former one
        gl::DepthFunc(gl::LESS);

        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, (GL_VERTEX_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       mem::transmute(&GL_VERTEX_DATA[0]), gl::STATIC_DRAW);

        gl::GenBuffers(1, &mut color_buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, color_buffer);
        gl::BufferData(gl::ARRAY_BUFFER, (GL_COLOR_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       mem::transmute(&GL_COLOR_DATA[0]), gl::STATIC_DRAW);
    }

    let vertex_shader = compile_shader("shaders/vertex.glsl", gl::VERTEX_SHADER);
    let fragment_shader = compile_shader("shaders/fragment.glsl", gl::FRAGMENT_SHADER);
    let program = link_program(vertex_shader, fragment_shader);
    let matrix_id = unsafe { gl::GetUniformLocation(program, CString::new("mvp").unwrap().as_ptr()) };

    while game.running {
        for event in window.poll_events() {
            match event {
                glutin::Event::Closed => {
                    game.running = false;
                },
                _ => ()
            };
        }

        let aspect = {
            if let Some((width, height)) = window.get_inner_size_pixels() {
                width as f32 / height as f32
            } else {
                4.0 / 3.0
            }
        };

        let projection = Matrix4::from(PerspectiveFov {
            fovy: cgmath::Rad::from(cgmath::Deg{ s: 90.0 }),
            aspect: aspect,
            near: 0.1,
            far: 128.0,
        });

        let view = Matrix4::look_at(
            Point3::new(4.0, 3.0, -3.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0)
        );

        let model_view_projection = projection * view * Matrix4::one();

        unsafe {

            gl::ClearColor(0.0, 0.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::UseProgram(program);

            gl::UniformMatrix4fv(matrix_id, 1, gl::FALSE, model_view_projection.as_ptr());

            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());

            gl::EnableVertexAttribArray(1);
            gl::BindBuffer(gl::ARRAY_BUFFER, color_buffer);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());

            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }

        window.swap_buffers();
    }
}

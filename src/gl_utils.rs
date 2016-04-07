extern crate gl;

use gl::types::*;
use std::mem;
use std::ptr;
use std::str;
use std::fs;
use std::ffi::CString;
use std::slice;

pub fn compile_shader(path: &str, ty: GLenum) -> GLuint {
    use std::io::Read;
    let mut src = String::new();
    let mut file = fs::File::open(path).unwrap_or_else(|err| panic!("Unable to load shader {}: {}", path, err));
    file.read_to_string(&mut src).unwrap_or_else(|err| panic!("Unable to load shader {}: {}", path, err));

    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("{}", str::from_utf8(&buf).ok().expect("ShaderInfoLog not valid utf8"));
        }
    }
    shader
}

pub fn link_program(vs: GLuint, fs: GLuint) -> GLuint { unsafe {
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
        panic!("{}", str::from_utf8(&buf).ok().expect("ProgramInfoLog not valid utf8"));
    }
    program
}}

pub fn init_vertex(program: u32, data: &[f32]){ unsafe{
    let mut vao = 0;
    let mut vbo = 0;
    // Create Vertex Array Object
    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);

    // Create a Vertex Buffer Object and copy the vertex data to it
    gl::GenBuffers(1, &mut vbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    gl::BufferData(gl::ARRAY_BUFFER,
                   (data.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                   mem::transmute(&data[0]),
                   gl::STATIC_DRAW);

    // Use shader program
    gl::UseProgram(program);
    gl::BindFragDataLocation(program, 0, CString::new("out_color").unwrap().as_ptr());

    // Specify the layout of the vertex data
    let pos_attr = gl::GetAttribLocation(program, CString::new("position").unwrap().as_ptr());
    gl::EnableVertexAttribArray(pos_attr as GLuint);
    gl::VertexAttribPointer(pos_attr as GLuint, 2, gl::FLOAT, gl::FALSE , 0, ptr::null());
}}

pub fn generate_buffers((sx, sy) : (u32, u32)) -> (u32, u32){ unsafe{
    let mut buf = [0u32, 0u32];
    gl::GenBuffers(2, &mut buf[0] as *mut u32);
    gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, buf[0]);
    gl::BufferData(gl::PIXEL_UNPACK_BUFFER, (sx * sy * 4) as i64, ptr::null(), gl::STREAM_DRAW);
    gl::ClearBufferData(gl::PIXEL_UNPACK_BUFFER, gl::RGBA8, gl::RGBA, gl::BYTE, mem::transmute(&0u32));
    gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, buf[1]);
    gl::BufferData(gl::PIXEL_UNPACK_BUFFER, (sx * sy * 4) as i64, ptr::null(), gl::STREAM_DRAW);
    gl::ClearBufferData(gl::PIXEL_UNPACK_BUFFER, gl::RGBA8, gl::RGBA, gl::BYTE, mem::transmute(&0u32));
    (buf[1], buf[0])
}}

pub fn map_buffer(size: usize) -> &'static mut[u8]{ unsafe{
    let pbo_p = gl::MapBuffer(gl::PIXEL_UNPACK_BUFFER, gl::WRITE_ONLY);
    slice::from_raw_parts_mut(mem::transmute(pbo_p), size)
}}

pub fn swap_buffer((buf_now, buf_next) : (u32, u32), (sx, sy) : (u32, u32)) -> (u32, u32){ unsafe{
    gl::UnmapBuffer(gl::PIXEL_UNPACK_BUFFER);
    gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, sx as i32, sy as i32, gl::RGBA, gl::UNSIGNED_INT_8_8_8_8_REV, ptr::null());
    gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, buf_next);
    (buf_next, buf_now)
}}

pub fn generate_texture((sx, sy) : (u32, u32)) -> u32{ unsafe{
    let mut tex: u32 = 0;
    gl::GenTextures(1, &mut tex);
    gl::BindTexture(gl::TEXTURE_2D, tex);
    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA8 as i32, sx as i32, sy as i32, 0, gl::RGBA, gl::UNSIGNED_INT_8_8_8_8_REV, ptr::null());
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    gl::ActiveTexture(tex);
    tex
}}

pub fn rescale_buffers((sx, sy) : (u32, u32), (buf_now, buf_next) : (u32, u32)){ unsafe{
    gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, buf_now);
    gl::BufferData(gl::PIXEL_UNPACK_BUFFER, (sx * sy * 4) as i64, ptr::null(), gl::STREAM_DRAW);
    gl::ClearBufferData(gl::PIXEL_UNPACK_BUFFER, gl::RGBA8, gl::RGBA, gl::BYTE, mem::transmute(&0u32));
    gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, buf_next);
    gl::BufferData(gl::PIXEL_UNPACK_BUFFER, (sx * sy * 4) as i64, ptr::null(), gl::STREAM_DRAW);
    gl::ClearBufferData(gl::PIXEL_UNPACK_BUFFER, gl::RGBA8, gl::RGBA, gl::BYTE, mem::transmute(&0u32));
}}

pub fn rescale_texture((sx, sy) : (u32, u32)){ unsafe{
    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA8 as i32, sx as i32, sy as i32, 0, gl::RGBA, gl::UNSIGNED_INT_8_8_8_8_REV, ptr::null());
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
}}

pub fn _get_uniform(program: u32, name: &str) -> i32{ unsafe{
    gl::GetUniformLocation(program, CString::new(name).unwrap().as_ptr())
}}

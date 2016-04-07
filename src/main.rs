#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![feature(float_extras)]
#![feature(asm)]
#![feature(zero_one)]
#![feature(test)]

extern crate sdl2;
extern crate gl;
extern crate time;

use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::cmp;

mod gl_utils;
use gl_utils::*;
mod worker;
use worker::*;
mod f8_120mod;
use f8_120mod::*;

use time::*;
use sdl2::event::{Event, WindowEventId};

const VERTEX_DATA: [f32; 8] = [
    -1.0, -1.0,
     1.0, -1.0,
    -1.0,  1.0,
     1.0,  1.0,
];

const THREADS: usize = 16;
const JOBS: usize = 128;

/*
const TEXTURE: [u8; 48] = [
9, 1, 47,
4, 4, 73,
0, 7, 100,
12, 44, 138,
24, 82, 177,
57, 125, 209,
134, 181, 229,
211, 236, 248,
241, 233, 191,
248, 201, 95,
255, 170, 0,
204, 128, 0,
153, 87, 0,
106, 52, 3,
66, 30, 15,
25, 7, 26,
];
*/

static TEXTURE: [u8; 36] = [
255, 0, 0,
127, 127, 127,
255, 255, 0,
127, 127, 127,
0, 255, 0,
127, 127, 127,
0, 255, 255,
127, 127, 127,
0, 0, 255,
127, 127, 127,
255, 0, 255,
127, 127, 127,
];


fn main() {
    let ctx = sdl2::init().unwrap_or_else(|err| panic!("Unable to initialize sdl2: {}", err));
    let video = ctx.video().unwrap_or_else(|err| panic!("Unable to initialize sld2 video: {}", err));
    video.gl_load_library_default().unwrap_or_else(|err| panic!("Unable to load gl library: {}", err));
    let gl_attr = video.gl_attr();
    gl_attr.set_double_buffer(false);
    let window =   video.window("Mandelrust", 800, 600)
                        .position_centered()
                        .resizable()
                        .opengl()
                        .build()
                        .unwrap_or_else(|err| panic!("Unable to create window: {}", err));
    let _glctx = window.gl_create_context().unwrap_or_else(|err| panic!("Unable to create gl context: {}", err));
    gl::load_with(|name| video.gl_get_proc_address(name));

    let vs = compile_shader("src/shader.vert", gl::VERTEX_SHADER);
    let fs = compile_shader("src/shader.frag", gl::FRAGMENT_SHADER);
    let program = link_program(vs, fs);

    init_vertex(program, &VERTEX_DATA);

    let mut events = ctx.event_pump();
    let mut last = SteadyTime::now();
    let mut fps = 0;

    let mut window_size: (u32, u32) = window.size();
    let mut scale: f32 = 1.0;
    let mut center: (f32, f32) = (0.0, 0.0);

    let mut buffers = generate_buffers(window_size);
    let _tex = generate_texture(window_size);

    let barrier = Arc::new(Barrier::new(THREADS + 1));
    let jobs = Arc::new(Mutex::new(Vec::<Job>::new()));

    for _ in 0 .. THREADS {
        let jobs_clone = jobs.clone();
        let barrier_clone = barrier.clone();
        thread::spawn(move || {
            let mut worker = Worker::new(barrier_clone);
            worker.run(jobs_clone);
        });
    }

    'main : loop {
        for event in events.as_mut().unwrap().poll_iter() {
            match event {
                Event::Quit{..} => break 'main,
                Event::Window{  win_event_id: WindowEventId::SizeChanged, data1: x, data2: y, .. } => unsafe{
                    window_size = (x as u32, y as u32);
                    gl::Viewport(0,0,x,y);
                    rescale_buffers(window_size, buffers);
                    rescale_texture(window_size);
                },
                Event::MouseWheel{ x, y, ..} => {
                    scale *=  1.0 - ( x + y ) as f32 * 0.2;
                },
                Event::MouseMotion{ mousestate, xrel, yrel, ..} if mousestate.left() => {
                    center.0 += xrel as f32 / window_size.0 as f32 * scale * 2.0;
                    center.1 -= yrel as f32 / window_size.0 as f32 * scale * 2.0;
                },
                _   => continue
            }
        }

        fps += 1;
        if SteadyTime::now() - last >= Duration::seconds(1) {
            println!("{} FPS", fps);
            last = SteadyTime::now();
            fps = 0;
        }

        unsafe{
            barrier.wait();
            let pbo = map_buffer((window_size.0 * window_size.1 * 4) as usize);
            {
                let mut jobs = jobs.lock().unwrap();
                let window_size = (window_size.0 as usize, window_size.1 as usize);
                let job_heigth = cmp::max(window_size.1 / JOBS, 1);
                let mut row = 0;
                let chunk_size = job_heigth * 4 * window_size.0;
                for slice in pbo.chunks_mut( chunk_size ) {
                    jobs.push(Job::new(
                        slice,
                        row,
                        window_size.0,
                        window_size.1,
                        f8_120::from(scale),
                        (f8_120::from(center.0), f8_120::from(center.1)),
                    ));
                    row += job_heigth;
                }
            }
            barrier.wait();
            buffers = swap_buffer(buffers, window_size);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
            gl::Flush();
            window.gl_swap_window();
        }
    }
}

extern crate time;
use time::*;
use std::f32::consts::PI as PI;
use std::sync::{Arc, Barrier, Mutex};
use std::ops::*;

use fixed::*;
use super::TEXTURE;

pub struct Job {
    pbo: &'static mut[u8],
    row: usize,
    width: usize,
    heigth: usize,
    scale: f8_120,
    center: (f8_120, f8_120),
}

impl Job {
    pub fn new(pbo: &'static mut[u8], row: usize, width: usize, heigth: usize, scale: f8_120, center: (f8_120, f8_120)) -> Job{
        Job{
            pbo: pbo,
            row: row,
            width: width,
            heigth: heigth,
            scale: scale,
            center: center,
        }
    }
}

pub struct Worker {
    barrier: Arc<Barrier>,
}

impl Worker {
    pub fn new( barrier: Arc<Barrier>) -> Worker {
        Worker{
            barrier: barrier,
        }
    }

    pub fn run( &mut self, jobs: Arc<Mutex<Vec<Job>>> ){
        loop{
            let job = jobs.lock().unwrap().pop();
            match job {
                Some(job) => self.do_job(job),
                None => {
                    self.barrier.wait();
                    self.barrier.wait();
                    continue;
                }
            };
        }
    }

    fn do_job( &mut self, job: Job) {
        use std::num::Zero;
        let offset = now().tm_nsec as f32 / (1_000_000_000.0) * PI * 2.0;
        let mut x = -_1;
        let mut y = f8_120::from(((job.row as f64/job.heigth as f64) - 0.5) * 2.0);
        let step_x = f8_120::from(2.0/job.width as f64);
        let step_y = f8_120::from(2.0/job.heigth as f64);
        let max = 100;

        for row in job.pbo.chunks_mut(4*job.width) {
            for pixel in row.chunks_mut(4) {
                {
                    let x = -job.center.0 + x * job.scale;
                    let y = -job.center.1 + y * job.scale;
                    
                    let i = Self::escape_time(x, y, max);

                    if i as i32 == max {
                        pixel[0] = 0;
                        pixel[1] = 0;
                        pixel[2] = 0;
                        pixel[3] = 255;
                    }else{
                        let color = interpolate(&TEXTURE, i);
                        pixel[0] = color.0;
                        pixel[1] = color.1;
                        pixel[2] = color.2;
                        pixel[3] = 255;
                    }
                }
                x = x + step_x;
            }
            x = -_1;
            y = y + step_y;
        }
    }
    
    fn partial_et<T>(x: T, y: T, max_i: i32, max_xy: T, two: T, mut x1: T, mut y1: T, mut i: i32) -> (T, T, i32)
        where T: PartialOrd + Add<Output=T> + Sub<Output=T> + Mul<Output=T> + Copy{
        let mut xx = x1 * x1;
        let mut yy = y1 * y1;
        while xx + yy <= max_xy && i < max_i {
            let xtemp = xx - yy + x;
            y1 = two*x1*y1 + y;
            x1 = xtemp;
            i += 1;
            xx = x1 * x1;
            yy = y1 * y1;
        }
        (x1, y1, i)
    }
    
    fn escape_time(x: f8_120, y: f8_120, max: i32) -> f64{
        
        let (x1, y1, i) = Self::partial_et(x, y, max, _1, _2, x, y, 0);
        let (x1, y1, i) = Self::partial_et(f64::from(x), f64::from(y), max, 256.0 * 256.0, 2.0, f64::from(x1), f64::from(y1), i);
        
        let log_zn = (x1 * x1 + y1 * y1).ln() / 2.0;
        let nu = (log_zn / 2.0f64.ln()).ln() / 2.0f64.ln();
        let col = i as f64 + 1.0 - nu;
        
        //i as f32
        if i == max {
            max as f64
        }else{
            col
        }
    }
}

fn interpolate(tex: &[u8], color: f64) -> (u8, u8, u8){
    let fract = color.fract();
    let trunc = color.trunc() as usize;
    let n1 = trunc%(tex.len()/3)*3;
    let n2 = (trunc + 1)%(tex.len()/3)*3;
    let col1 = &tex[n1..n1+3];
    let col2 = &tex[n2..n2+3];
    (
        (col2[0] as f64 * fract + col1[0] as f64 * (1.0-fract)) as u8,
        (col2[1] as f64 * fract + col1[1] as f64 * (1.0-fract)) as u8,
        (col2[2] as f64 * fract + col1[2] as f64 * (1.0-fract)) as u8,
    )
}

const _0: f8_120 = f8_120{ words: (0b00000000_00000000000000000000000000000000000000000000000000000000, 0), sign: Sign::Neutral};
const _1: f8_120 = f8_120{ words: (0b00000001_00000000000000000000000000000000000000000000000000000000, 0), sign: Sign::Positive};
const _2: f8_120 = f8_120{ words: (0b00000010_00000000000000000000000000000000000000000000000000000000, 0), sign: Sign::Positive};
const _4: f8_120 = f8_120{ words: (0b00000100_00000000000000000000000000000000000000000000000000000000, 0), sign: Sign::Positive};

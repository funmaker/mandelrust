extern crate time;
use time::*;
use std::f32::consts::PI as PI;
use std::sync::{Arc, Barrier, Mutex};

use f8_120mod::*;
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

        for row in job.pbo.chunks_mut(4*job.width) {
            for pixel in row.chunks_mut(4) {
                {
                    let x = -job.center.0 + x * job.scale;
                    let y = -job.center.1 + y * job.scale;
                    let mut x1 = x;
                    let mut y1 = y;
                    let mut i = 0;
                    let mut xx = x1 * x1;
                    let mut yy = y1 * y1;
                    while xx + yy < _4 && i < 100 {
                        let xtemp = xx - yy + x;
                        y1 = _2*x1*y1 + y;
                        x1 = xtemp;
                        i += 1;
                        xx = x1 * x1;
                        yy = y1 * y1;
                    }

                    if i == 100 {
                        pixel[0] = 0;
                        pixel[1] = 0;
                        pixel[2] = 0;
                        pixel[3] = 255;
                    }else{
                        pixel[0] = TEXTURE[(i%(TEXTURE.len()/3))*3 + 0];
                        pixel[1] = TEXTURE[(i%(TEXTURE.len()/3))*3 + 1];
                        pixel[2] = TEXTURE[(i%(TEXTURE.len()/3))*3 + 2];
                        pixel[3] = 255;
                    }
                }
                x = x + step_x;
            }
            x = -_1;
            y = y + step_y;
        }
    }
}

const _0: f8_120 = f8_120{ words: (0b00000000_00000000000000000000000000000000000000000000000000000000, 0), sign: Sign::Neutral};
const _1: f8_120 = f8_120{ words: (0b00000001_00000000000000000000000000000000000000000000000000000000, 0), sign: Sign::Positive};
const _2: f8_120 = f8_120{ words: (0b00000010_00000000000000000000000000000000000000000000000000000000, 0), sign: Sign::Positive};
const _4: f8_120 = f8_120{ words: (0b00000100_00000000000000000000000000000000000000000000000000000000, 0), sign: Sign::Positive};

extern crate futures;
extern crate futures_cpupool;
extern crate image;
extern crate num_cpus;

use self::futures_cpupool::{Builder, CpuPool};
use self::futures::future::join_all;
use std::io::Write;
use std::sync::mpsc::channel;

struct Point {
    x: u32,
    y: u32,
    value: u8,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Mandlebrot {
    pool: CpuPool,
}

impl Mandlebrot {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let cpus = num_cpus::get();
        let pool = Builder::new().pool_size(cpus).create();
        Self::new_with_pool(pool)
    }

    pub fn new_with_pool(pool: CpuPool) -> Self {
        Self { pool: pool }
    }

    pub fn generate<W: Write>(&self, width: u32, height: u32, out: &mut W) {
        let (tx, rx) = channel();
        let threads = (0..width)
            .map(|x| {
                let tx = tx.clone();
                self.pool.spawn_fn(move || {
                    for y in 0..height {
                        let value = generate_pixel(x, y, width as f64, height as f64);
                        let pt = Point {
                            x: x,
                            y: y,
                            value: value,
                        };
                        tx.send(pt).unwrap();
                    }
                    let result: Result<(), ()> = Ok(());
                    result
                })
            })
            .collect::<Vec<_>>();

        let mut img: image::GrayImage = image::ImageBuffer::new(width, height);

        for _ in 0..width * height {
            let pt = rx.recv().unwrap();
            img.put_pixel(pt.x, pt.y, image::Luma([pt.value as u8]))
        }

        let _ = join_all(threads);

        // We must indicate the image's color type and what format to save as
        image::ImageLuma8(img).save(out, image::PNG).unwrap();
    }
}

fn generate_pixel(i: u32, j: u32, width: f64, height: f64) -> u8 {
    let xi = norm(i as f64, width as f64, -1.0, 2.0);
    let yi = norm(j as f64, height as f64, -1.0, 1.0);

    const COMPLEXITY: f64 = 1024.0;
    let (mut x, mut y) = (0., 0.);
    let mut i = 0;
    while (x * x + y * y < COMPLEXITY) && i < 1000 {
        let (xm, ym) = (x * x - y * y + xi, 2.0 * x * y + yi);
        x = xm;
        y = ym;
        i = i + 1;
    }
    x as u8
}

fn norm(x: f64, total: f64, min: f64, max: f64) -> f64 {
    (max - min) * x / total - max
}

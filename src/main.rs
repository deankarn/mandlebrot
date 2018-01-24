extern crate futures;
extern crate futures_cpupool;
extern crate image;
extern crate num_cpus;

use std::fs::File;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use futures::future::{join_all, Future};
use futures_cpupool::{Builder, CpuPool};

const WIDTH: u32 = 2048;
const HEIGHT: u32 = 2048;

struct Point {
    x: u32,
    y: u32,
    value: u8,
}

fn main() {
    let pool = Builder::new().pool_size(2048).create();
    let (tx, rx) = channel();
    let threads = (0..WIDTH)
        .map(|x| {
            let tx = tx.clone();
            pool.spawn_fn(move || {
                for y in 0..HEIGHT {
                    let value = generate_pixel(x, y);
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

    let mut img: image::GrayImage = image::ImageBuffer::new(WIDTH, HEIGHT);

    for _ in 0..WIDTH * HEIGHT {
        let pt = rx.recv().unwrap();
        img.put_pixel(pt.x, pt.y, image::Luma([pt.value as u8]))
    }

    let _ = join_all(threads);
    // for h in threads {
    //     let _ = h.join();
    // }
    // Save the image as “fractal.png”
    let ref mut fout = File::create("out.png").unwrap();

    // We must indicate the image's color type and what format to save as
    image::ImageLuma8(img).save(fout, image::PNG).unwrap();
}

fn generate_pixel(i: u32, j: u32) -> u8 {
    let xi = norm(i as f64, WIDTH as f64, -1.0, 2.0);
    let yi = norm(j as f64, HEIGHT as f64, -1.0, 1.0);

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

// fn set_pixel(i: u32, j: u32, pixel: &mut image::Luma<u8>) {
//     let xi = norm(i as f64, WIDTH as f64, -1.0, 2.0);
//     let yi = norm(j as f64, HEIGHT as f64, -1.0, 1.0);

//     const COMPLEXITY: f64 = 1024.0;
//     let (mut x, mut y) = (0., 0.);
//     let mut i = 0;
//     while (x * x + y * y < COMPLEXITY) && i < 1000 {
//         let (xm, ym) = (x * x - y * y + xi, 2.0 * x * y + yi);
//         x = xm;
//         y = ym;
//         i = i + 1;
//     }
//     *pixel = image::Luma([x as u8]);
// }

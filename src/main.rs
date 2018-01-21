extern crate image;

use std::fs::File;

const WIDTH:u32 =2048;
const HEIGHT:u32 =2048;

fn main() {
    let mut buff = image::ImageBuffer::new(WIDTH,HEIGHT);

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in buff.enumerate_pixels_mut() {
        // Create an 8bit pixel of type Luma and value i
        // and assign in to the pixel at position (x, y)
        set_pixel(x,y,pixel)
    }

    // Save the image as “fractal.png”
    let ref mut fout = File::create("out.png").unwrap();

    // We must indicate the image's color type and what format to save as
    image::ImageLuma8(buff).save(fout, image::PNG).unwrap();
}

fn set_pixel(i:u32,j: u32, pixel:&mut image::Luma<u8>){
    let xi = norm(i as f64, WIDTH as f64, -1.0, 2.0);
    let yi = norm(j as f64, HEIGHT as f64, -1.0, 1.0);

    const COMPLEXITY:f64 = 1024.0;
    let  (mut x,  mut y) = (0., 0.);
    let mut i = 0;
    while (x*x+y*y < COMPLEXITY) && i < 1000 {
        let (xm, ym) = (x*x-y*y+xi,2.0*x*y+yi);
        x = xm;
        y = ym;
        i = i+1;
    }
    *pixel = image::Luma([ x as u8]);
}

fn norm(x:f64,total:f64,min:f64,max:f64) -> f64 {
    (max-min)*x/total - max
}

// // pixel returns the color of a Mandelbrot fractal at the given point.
// func pixel(i, j, width, height int) color.Color {
// 	// Play with this constant to increase the complexity of the fractal.
// 	// In the justforfunc.com video this was set to 4.
// 	const complexity = 1024

// 	xi := norm(i, width, -1.0, 2)
// 	yi := norm(j, height, -1, 1)

// 	const maxI = 1000
// 	x, y := 0., 0.

// 	for i := 0; (x*x+y*y < complexity) && i < maxI; i++ {
// 		x, y = x*x-y*y+xi, 2*x*y+yi
// 	}

// 	return color.Gray{uint8(x)}
// }

// func norm(x, total int, min, max float64) float64 {
// 	return (max-min)*float64(x)/float64(total) - max
// }
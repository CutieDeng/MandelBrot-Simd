#![feature(portable_simd, iter_array_chunks)]

use std::time::Instant;
use std::simd::{f32x64, SimdPartialOrd, mask32x64};

struct Complex {
    re: Vec<f32x64>, 
    im: Vec<f32x64>, 
}

impl Complex {
    fn new(x_col: usize, y_col: usize, x_low: f32, x_step: f32, y_low: f32, y_step: f32) -> Self {
        let length = x_col.checked_mul(y_col).unwrap(); 
        let mut re = vec![f32x64::splat(0f32); length]; 
        let mut im = vec![f32x64::splat(0f32); length]; 
        for i in 0..x_col {
            for j in 0..y_col {
                let index = i.checked_mul(y_col).unwrap().checked_add(j).unwrap(); 
                let ref mut r = re[index];
                for k in 0..64 {
                    r[k] = x_low + x_step * i as f32 + ( (k % 8) as f32 ) / 8f32 * x_step;  
                }
                let ref mut r = im[index]; 
                for k in 0..64 {
                    r[k] = y_low + y_step * j as f32 + ( (k / 8) as f32 ) / 8f32 * y_step; 
                } 
            }
        } 
        Complex { re, im } 
    }
} 

fn mandelbrot(re: &f32x64, im: &f32x64, rst: &mut f32x64) { 
    let mut z_re = f32x64::splat(0f32); 
    let mut z_im = f32x64::splat(0f32); 

    let mut zs_re; 
    let mut zs_im; 

    let mut mask; 
    let mut allow_assign = mask32x64::splat(true); 
    let mut zs; 

    let mut it = 0; 
    
    *rst = f32x64::splat(255f32); 

    loop {
        zs_re = z_re * z_re; 
        zs_re -= z_im * z_im; 
        zs_im = z_re * z_im; 
        zs_im += zs_im; 
        zs_re += re; 
        zs_im += im; 
             
        z_re = zs_re; 
        z_im = zs_im; 
        zs = zs_re * zs_re; 
        zs += zs_im * zs_im; 

        mask = zs.simd_ge(f32x64::splat(32f32)); 
        mask = mask & allow_assign; 
        allow_assign = allow_assign & !mask; 

        if !allow_assign.any() || it == 255 { 
            break; 
        }
        if mask.any() {
            *rst = mask.select(f32x64::splat(it as f32), *rst); 
        }
        it += 1;
    }
}

fn main() {
    let start_ins = Instant::now(); 
    const X_COL: usize = 240; 
    const Y_COL: usize = 135; 
    const X_LOW: f32 = -2.65; 
    const Y_LOW: f32 = -1.25; 
    const X_UP: f32 = X_LOW + 4.0; 
    const Y_UP: f32 = Y_LOW + 2.5; 
    const X_STEP: f32 = (X_UP - X_LOW) / X_COL as f32; 
    const Y_STEP: f32 = (Y_UP - Y_LOW) / Y_COL as f32; 
    let mut c = Complex::new(X_COL, Y_COL, X_LOW, X_STEP, Y_LOW, Y_STEP); 
    let mut rst : Vec<f32x64> = vec![f32x64::splat(0f32); X_COL * Y_COL]; 
    let i = c.im.iter_mut().zip(c.re.iter_mut());
    for (idx, (im, re)) in i.enumerate() {
        let ref mut r = rst[idx]; 
        mandelbrot(re, im, r); 
    } 
    // create image buffer 
    let width = X_COL * 8; 
    let width = width as u32; 
    let height = Y_COL * 8;
    let height = height as u32; 
    let mut imgbuf = image::ImageBuffer::new(width, height); 
    for r in imgbuf.enumerate_rows_mut() {
        for c in r.1.array_chunks::<8>() {
            let y = c[0].1 as usize; 
            let yidx = y / 8; 
            let yelse = y % 8; 
            for p in c {
                let x = p.0 as usize; 
                let xidx = x / 8; 
                let xelse = x % 8; 
                let index = xidx.checked_mul(Y_COL).unwrap().checked_add(yidx).unwrap(); 
                let r = &rst[index]; 
                let subindex = yelse * 8 + xelse; 
                let subindex = subindex as usize; 
                let r = r[subindex]; 
                *p.2 = image::Rgb(color((2.0 * r / 256.0 + 0.5) % 1.0)); 
            } 
        }
    }
    imgbuf.save("mandelbrot.png").unwrap(); 
    let end_ins = Instant::now(); 
    println!("time: {:?}", end_ins - start_ins); 
}

pub fn color(t: f32) -> [u8; 3] {
    let a = (0.5, 0.5, 0.5);
    let b = (0.5, 0.5, 0.5);
    let c = (1.0, 1.0, 1.0);
    let d = (0.0, 0.10, 0.20);
    let r = b.0 * (6.28318 * (c.0 * t + d.0)).cos() + a.0;
    let g = b.1 * (6.28318 * (c.1 * t + d.1)).cos() + a.1;
    let b = b.2 * (6.28318 * (c.2 * t + d.2)).cos() + a.2;
    [(255.0 * r) as u8, (255.0 * g) as u8, (255.0 * b) as u8]
}
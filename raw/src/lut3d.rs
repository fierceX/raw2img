use rayon::prelude::*;
use std::{fs::File, io::{self, BufRead, BufReader}};

const MAX_LEVEL: usize = 65; // 根据实际情况定义MAX_LEVEL

#[derive(Debug, Clone, Copy)]
struct RGBVec {
    r: f32,
    g: f32,
    b: f32,
}

#[derive(Debug)]
pub struct LUT3DContext {
    lut: Vec<Vec<Vec<RGBVec>>>,
    lutsize: usize,
    step: usize,
}


impl LUT3DContext {
    fn new(lutsize: usize) -> Self {
        LUT3DContext {
            lut: vec![
                vec![
                    vec![
                        RGBVec {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0
                        };
                        lutsize
                    ];
                    lutsize
                ];
                lutsize
            ],
            lutsize,
            step: 3 as usize,
        }
    }
}

pub fn parse_cube(filename: &str) -> io::Result<LUT3DContext> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file).lines();
    // let mut lut3d = None;
    let mut min = [0.0f32; 3];
    let mut max = [1.0f32; 3];
    let mut lutsize = 0;
    let mut rgbs = Vec::new();

    for line in reader {
        let line = line?;
        if line.starts_with("LUT_3D_SIZE ") {
            let size = line[12..].parse::<usize>().unwrap();
            if size < 2 || size > MAX_LEVEL {
                println!("Too large or invalid 3D LUT size");
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid LUT size",
                ));
            }
            lutsize = size;
            continue;
        }
        if line.starts_with("DOMAIN_") {
            if line.starts_with("DOMAIN_MIN ") {
                let _m: Vec<&str> = line.split_whitespace().collect();
                min[0] = _m[1].parse::<f32>().unwrap();
                min[1] = _m[2].parse::<f32>().unwrap();
                min[2] = _m[3].parse::<f32>().unwrap();
            } else if line.starts_with("DOMAIN_MAX ") {
                let _m: Vec<&str> = line.split_whitespace().collect();
                max[0] = _m[1].parse::<f32>().unwrap();
                max[1] = _m[2].parse::<f32>().unwrap();
                max[2] = _m[3].parse::<f32>().unwrap();
            }
            continue;
        }

        let rgb_values: Vec<&str> = line.split_whitespace().collect();
        if rgb_values.len() == 3 {
            let r;
            let g;
            let b;
            match rgb_values[0].parse::<f32>() {
                Ok(_r) => {
                    r = _r;
                }
                Err(_e) => {
                    continue;
                }
            }
            match rgb_values[1].parse::<f32>() {
                Ok(_g) => {
                    g = _g;
                }
                Err(_e) => {
                    continue;
                }
            }
            match rgb_values[2].parse::<f32>() {
                Ok(_b) => {
                    b = _b;
                }
                Err(_e) => {
                    continue;
                }
            }
            rgbs.push(RGBVec { r, g, b });
        }
    }
    let mut n = 0;
    let mut lut3d_context = LUT3DContext::new(lutsize);
    for k in 0..lutsize {
        for j in 0..lutsize {
            for i in 0..lutsize {
                let vec = &mut lut3d_context.lut[i][j][k];

                vec.r = rgbs[n].r * (max[0] - min[0]);
                vec.g = rgbs[n].g * (max[1] - min[1]);
                vec.b = rgbs[n].b * (max[2] - min[2]);
                n = n + 1;
            }
        }
    }
    Ok(lut3d_context)
}

fn interp_tetrahedral(lut3d: &LUT3DContext, s: RGBVec) -> RGBVec {
    let prev = [s.r as i32, s.g as i32, s.b as i32];
    let next = [
        std::cmp::min(s.r as i32 + 1, lut3d.lutsize as i32 - 1),
        std::cmp::min(s.g as i32 + 1, lut3d.lutsize as i32 - 1),
        std::cmp::min(s.b as i32 + 1, lut3d.lutsize as i32 - 1),
    ];

    let d = RGBVec {
        r: s.r - prev[0] as f32,
        g: s.g - prev[1] as f32,
        b: s.b - prev[2] as f32,
    };
    let c000 = lut3d.lut[prev[0] as usize][prev[1] as usize][prev[2] as usize];
    let c111 = lut3d.lut[next[0] as usize][next[1] as usize][next[2] as usize];
    let mut c = RGBVec {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
    if d.r > d.g {
        if d.g > d.b {
            let c100 = lut3d.lut[next[0] as usize][prev[1] as usize][prev[2] as usize];
            let c110 = lut3d.lut[next[0] as usize][next[1] as usize][prev[2] as usize];
            c.r = (1 as f32 - d.r) * c000.r
                + (d.r - d.g) * c100.r
                + (d.g - d.b) * c110.r
                + (d.b) * c111.r;
            c.g = (1 as f32 - d.r) * c000.g
                + (d.r - d.g) * c100.g
                + (d.g - d.b) * c110.g
                + (d.b) * c111.g;
            c.b = (1 as f32 - d.r) * c000.b
                + (d.r - d.g) * c100.b
                + (d.g - d.b) * c110.b
                + (d.b) * c111.b;
        } else if d.r > d.b {
            let c100 = lut3d.lut[next[0] as usize][prev[1] as usize][prev[2] as usize];
            let c101 = lut3d.lut[next[0] as usize][prev[1] as usize][next[2] as usize];
            c.r = (1 as f32 - d.r) * c000.r
                + (d.r - d.b) * c100.r
                + (d.b - d.g) * c101.r
                + (d.g) * c111.r;
            c.g = (1 as f32 - d.r) * c000.g
                + (d.r - d.b) * c100.g
                + (d.b - d.g) * c101.g
                + (d.g) * c111.g;
            c.b = (1 as f32 - d.r) * c000.b
                + (d.r - d.b) * c100.b
                + (d.b - d.g) * c101.b
                + (d.g) * c111.b;
        } else {
            let c001 = lut3d.lut[prev[0] as usize][prev[1] as usize][next[2] as usize];
            let c101 = lut3d.lut[next[0] as usize][prev[1] as usize][next[2] as usize];
            c.r = (1 as f32 - d.b) * c000.r
                + (d.b - d.r) * c001.r
                + (d.r - d.g) * c101.r
                + (d.g) * c111.r;
            c.g = (1 as f32 - d.b) * c000.g
                + (d.b - d.r) * c001.g
                + (d.r - d.g) * c101.g
                + (d.g) * c111.g;
            c.b = (1 as f32 - d.b) * c000.b
                + (d.b - d.r) * c001.b
                + (d.r - d.g) * c101.b
                + (d.g) * c111.b;
        }
    } else {
        if d.b > d.g {
            let c001 = lut3d.lut[prev[0] as usize][prev[1] as usize][next[2] as usize];
            let c011 = lut3d.lut[prev[0] as usize][next[1] as usize][next[2] as usize];
            c.r = (1 as f32 - d.b) * c000.r
                + (d.b - d.g) * c001.r
                + (d.g - d.r) * c011.r
                + (d.r) * c111.r;
            c.g = (1 as f32 - d.b) * c000.g
                + (d.b - d.g) * c001.g
                + (d.g - d.r) * c011.g
                + (d.r) * c111.g;
            c.b = (1 as f32 - d.b) * c000.b
                + (d.b - d.g) * c001.b
                + (d.g - d.r) * c011.b
                + (d.r) * c111.b;
        } else if d.b > d.r {
            let c010 = lut3d.lut[prev[0] as usize][next[1] as usize][prev[2] as usize];
            let c011 = lut3d.lut[prev[0] as usize][next[1] as usize][next[2] as usize];
            c.r = (1 as f32 - d.g) * c000.r
                + (d.g - d.b) * c010.r
                + (d.b - d.r) * c011.r
                + (d.r) * c111.r;
            c.g = (1 as f32 - d.g) * c000.g
                + (d.g - d.b) * c010.g
                + (d.b - d.r) * c011.g
                + (d.r) * c111.g;
            c.b = (1 as f32 - d.g) * c000.b
                + (d.g - d.b) * c010.b
                + (d.b - d.r) * c011.b
                + (d.r) * c111.b;
        } else {
            let c010 = lut3d.lut[prev[0] as usize][next[1] as usize][prev[2] as usize];
            let c110 = lut3d.lut[next[0] as usize][next[1] as usize][prev[2] as usize];
            c.r = (1 as f32 - d.g) * c000.r
                + (d.g - d.r) * c010.r
                + (d.r - d.b) * c110.r
                + (d.b) * c111.r;
            c.g = (1 as f32 - d.g) * c000.g
                + (d.g - d.r) * c010.g
                + (d.r - d.b) * c110.g
                + (d.b) * c111.g;
            c.b = (1 as f32 - d.g) * c000.b
                + (d.g - d.r) * c010.b
                + (d.r - d.b) * c110.b
                + (d.b) * c111.b;
        }
    }
    c
}

fn interp_nearest(lut3d: &LUT3DContext, s: RGBVec) -> RGBVec {
    let v = lut3d.lut[(s.r + 0.5) as usize][(s.g + 0.5) as usize][(s.b + 0.5) as usize];
    v
}

fn clip_uint(a: f32) -> u8 {
    let b = a as i32;
    if b < 0 || b > 255 {
        0 // 由于Rust是强类型语言，不需要显式地写((~a) >> 31)
    } else {
        b as u8
    }
}

pub fn interp_8_tetrahedral(
    lut3d: LUT3DContext,
    indata: Vec<u8>,
    width: i32,
    colors: i32,
) -> Vec<u8> {
    let nbits = 8;
    let step = lut3d.step;
    let r = 0;
    let g = 1;
    let b = 2;
    // let a = 0;
    let mut outdata = vec![0; indata.len()];
    let scale = (1.0 / ((1 << nbits) - 1) as f32) * (lut3d.lutsize - 1) as f32;
    let linesize = width * colors;
    // let start = Instant::now();
    //     // 计算每个像素的索引
    outdata
        .par_chunks_mut(linesize as usize)
        .enumerate()
        .for_each(|(y, out_chunk)| {
            for x in (0..width * step as i32).step_by(step as usize) {
                let scaled_rgb = RGBVec {
                    r: indata[(y as i32 * linesize + x + r) as usize] as f32 * scale,
                    g: indata[(y as i32 * linesize + x + g) as usize] as f32 * scale,
                    b: indata[(y as i32 * linesize + x + b) as usize] as f32 * scale,
                };
                let vec = interp_tetrahedral(&lut3d, scaled_rgb);
                out_chunk[(x + r) as usize] = clip_uint(vec.r * ((1 << nbits) - 1) as f32);
                out_chunk[(x + g) as usize] = clip_uint(vec.g * ((1 << nbits) - 1) as f32);
                out_chunk[(x + b) as usize] = clip_uint(vec.b * ((1 << nbits) - 1) as f32);
            }
        });

    // let duration = start.elapsed();

    // // 输出运行时间
    // println!("运行时间: {:?}", duration);

    // let start = Instant::now();

    // for y in 0..height {
    //     for x in (0..width * step as i32).step_by(step) {
    //         let scaled_rgb = RGBVec {
    //             r: indata[(y * linesize + x + r) as usize] as f32 * scale,
    //             g: indata[(y * linesize + x + g) as usize] as f32 * scale,
    //             b: indata[(y * linesize + x + b) as usize] as f32 * scale,
    //         };
    //         let _vec = interp_tetrahedral(&lut3d, scaled_rgb);
    //         outdata[(y * linesize + x + r) as usize] =
    //             clip_uint(_vec.r * ((1 << nbits) - 1) as f32);
    //         outdata[(y * linesize + x + g) as usize] =
    //             clip_uint(_vec.g * ((1 << nbits) - 1) as f32);
    //         outdata[(y * linesize + x + b) as usize] =
    //             clip_uint(_vec.b * ((1 << nbits) - 1) as f32);
    //     }
    // }

    // let duration = start.elapsed();

    // // 输出运行时间
    // println!("运行时间: {:?}", duration);

    outdata
}

use std::ffi::{c_char, c_int, CString};
use std::os::macos::raw;
use std::{
    default,
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::Path,
};

use image::{DynamicImage, ExtendedColorType, ImageBuffer, ImageEncoder};
use img_parts::{jpeg::Jpeg, Bytes, ImageEXIF};
use libraw_rs_vendor::{
    libraw_close, libraw_data_t, libraw_dcraw_make_mem_image, libraw_dcraw_process, libraw_init,
    libraw_open_file, libraw_processed_image_t, libraw_unpack, LibRaw_errors_LIBRAW_SUCCESS,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use chrono::{TimeZone, Local};

use exif::experimental::Writer;
use exif::{Field, In, Tag, Value};

use jpeg_encoder;
use webp::Encoder;
mod lut3d;
mod img_frame;
use crate::img_frame::gen_frame_img;
use crate::lut3d::{interp_8_tetrahedral, parse_cube};

pub struct RawData {
    data: Vec<u8>,
    width: i32,
    height: i32,
    colors: i32,
    iso: f32,
    aperture: f32,
    shutter: f32,
    focal_len: u16,
    shooting_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Myexif {
    pub iso: f32,
    pub aperture: f32,
    pub shutter: f32,
    pub focal_len: u16,
    pub shooting_date:String,
}

fn exposure_shift(data: &[u8]) -> f32 {
    // let start = Instant::now();
    let mut v = 0.0;
    let (m, j) = data
        .par_iter()
        .filter(|&i| (*i > 20) && (*i < 220))
        .map(|&i| (i as i32, 1))
        .reduce(|| (0, 0), |acc, x| [acc.0 + x.0, acc.1 + x.1].into());

    // let duration = start.elapsed();

    // // 输出运行时间
    // println!("运行时间: {:?}", duration);

    // let start = Instant::now();

    // let mut m = 0;
    // let mut j = 0;
    // for i in data.iter(){
    //     if *i < 220 && *i > 20 {
    //         m += *i as i32;
    //         j += 1;
    //     }
    // }
    // let duration = start.elapsed();

    // // 输出运行时间
    // println!("运行时间: {:?}", duration);
    // m = m/j;

    let mm = m / j;
    if mm < 120 && mm > 60 {
        v = f32::powf(2.0, 110.0 / mm as f32);
        if 110 < mm {
            v = -v;
        }
    }
    v
}

fn generate_gamma_lut(lut: &mut [u8], gamma: f32) {
    println!("{:?}", gamma);
    for i in 0..=TBLN {
        lut[i] = ((i as f32 / 255.0).powf(1.0 / gamma) * 255.0).round() as u8;
    }
}

const TBLN: usize = 255;

fn generate_linear_lut(lut: &mut [u8], shift: f32, smooth: f32) {
    let x1: f32;
    let x2: f32 = TBLN as f32;
    let y1: f32;
    let y2: f32;

    let cstops = shift.ln() / 2.0f32.ln();
    let room = cstops * 2.0;
    let roomlin = 2.0f32.powf(room);
    x1 = (x2 + 1.0) / roomlin - 1.0;
    y1 = x1 * shift;
    y2 = x2 * (1.0 + (1.0 - smooth) * (shift - 1.0));
    let sq3x = (x1 * x1 * x2).powf(1.0 / 3.0);
    let b = (y2 - y1 + shift * (3.0 * x1 - 3.0 * sq3x)) / (x2 + 2.0 * x1 - 3.0 * sq3x);
    let a = (shift - b) * 3.0 * (x1 * x1).powf(1.0 / 3.0);
    let cc = y2 - a * x2.powf(1.0 / 3.0) - b * x2;

    for i in 0..=TBLN {
        let x = i as f32;
        let y = a * x.powf(1.0 / 3.0) + b * x + cc;
        if i < x1 as usize {
            lut[i] = (x * shift) as u8;
        } else {
            lut[i] = if y < 0.0 {
                0
            } else if y > TBLN as f32 {
                TBLN as u8
            } else {
                y as u8
            };
        }
    }
}

fn apply_exp_correction(image: &mut Vec<u8>, shift: f32, luttype: &str) {
    let mut lut = [0u8; TBLN + 1];
    let smooth = 0.9f32; // 示例值
    match luttype {
        "gamma" => {
            generate_gamma_lut(&mut lut, shift);
        }
        "linear" => {
            generate_linear_lut(&mut lut, shift, smooth);
        }
        _default => {
            generate_linear_lut(&mut lut, shift, smooth);
        }
    }

    for channel in image.iter_mut() {
        *channel = lut[*channel as usize];
    }
}

fn read_raw(input: &str, wb: bool, half_size: bool, exp_shift: f32, threshold: i32) -> RawData {
    let libraw_data: *mut libraw_data_t = unsafe { libraw_init(0) };
    let rust_var = 0;
    let input_c_str = CString::new(input).unwrap();
    let input_c_world: *const c_char = input_c_str.as_ptr() as *const c_char;
    unsafe {
        libraw_open_file(libraw_data, input_c_world);
        libraw_unpack(libraw_data);
        (*(libraw_data)).params.exp_correc = 1;
        (*(libraw_data)).params.exp_preser = 0.8;
        let iso = (*(libraw_data)).other.iso_speed;
        let aperture = (*(libraw_data)).other.aperture;
        let shutter = (*(libraw_data)).other.shutter;
        let timestamp = (*(libraw_data)).other.timestamp;
        let focal_len = (*(libraw_data)).lens.FocalLengthIn35mmFormat;
        if wb {
            (*(libraw_data)).params.use_auto_wb = 1;
        } else {
            (*(libraw_data)).params.use_camera_wb = 1;
        }
        if threshold > -1 {
            (*(libraw_data)).params.threshold = threshold as f32;
        } else {
            (*(libraw_data)).params.threshold = 256.0 * ((*(libraw_data)).other.iso_speed / 400.0);
        }
        if exp_shift >= -2.0 {
            (*(libraw_data)).params.exp_shift = f32::powf(2.0, exp_shift);
        } else {
            (*(libraw_data)).params.half_size = 1;
            libraw_dcraw_process(libraw_data);

            let ptr: *mut c_int = rust_var as *mut c_int;
            let img = libraw_dcraw_make_mem_image(libraw_data, ptr);
            let raw_data = (*img).data.as_ptr();
            let raw_size = (*img).data_size;
            let _width = (*img).width as i32;
            let _height = (*img).height as i32;
            let _colors = (*img).colors as i32;
            // 使用指针和大小创建一个 Rust 切片
            let _rawdata = std::slice::from_raw_parts(raw_data, raw_size as usize);
            let v = exposure_shift(_rawdata);
            (*(libraw_data)).params.exp_shift = v;
        }
        (*(libraw_data)).params.half_size = half_size as i32;
        libraw_dcraw_process(libraw_data);

        let ptr: *mut c_int = rust_var as *mut c_int;
        let img = libraw_dcraw_make_mem_image(libraw_data, ptr);
        let raw_data = (*img).data.as_ptr();
        let raw_size = (*img).data_size;
        let _width = (*img).width as i32;
        let _height = (*img).height as i32;
        let _colors = (*img).colors as i32;
        let datetime = Local.timestamp_opt(timestamp, 0).unwrap();

        // 使用指针和大小创建一个 Rust 切片
        let _rawdata = std::slice::from_raw_parts(raw_data, raw_size as usize);

        let rawdata = RawData {
            data: _rawdata.to_vec(),
            width: _width,
            height: _height,
            colors: _colors,
            iso,
            aperture,
            shutter,
            focal_len,
            shooting_date:datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        libraw_close(libraw_data);
        rawdata
    }
}

fn save(
    output: String,
    data: Vec<u8>,
    width: u32,
    height: u32,
    quality: i32,
    _exif: &Myexif,
    embed_exif:bool,
    font_file:&str
) -> Result<String, String> {
    let (data,width,height) = if font_file != ""{
        let exif_str = format!("{}mm f/{} 1/{}s ISO{}",_exif.focal_len,_exif.aperture,(1.0/_exif.shutter).round(),_exif.iso);
        gen_frame_img(data,width,height,&exif_str,false,true,None,font_file)
    }
    else{
        (data,width,height)
    };
    match output.split('.').last() {
        Some(suffix) if suffix == "webp" => {
            let encoder = Encoder::from_rgb(&data, width, height);
            let webp = encoder.encode(quality as f32);
            fs::write(output, &*webp).unwrap();
            Ok("aaa".to_string())
        }
        Some(suffix) if suffix == "jpg" => {
            {
                if embed_exif{
                    let mut buf_writer = BufWriter::new(std::io::Cursor::new(Vec::new()));
                    let mut encoder =
                        jpeg_encoder::Encoder::new(&mut buf_writer, quality.try_into().unwrap());
                    encoder.set_progressive(true);
                    encoder
                        .encode(
                            &data,
                            width.try_into().unwrap(),
                            height.try_into().unwrap(),
                            jpeg_encoder::ColorType::Rgb,
                        )
                        .unwrap();

                    let image_desc1 = Field {
                        tag: Tag::ExposureTime,
                        ifd_num: In::PRIMARY,
                        value: Value::Float(vec![_exif.shutter]),
                    };

                    let image_desc2 = Field {
                        tag: Tag::FNumber,
                        ifd_num: In::PRIMARY,
                        value: Value::Float(vec![_exif.aperture]),
                    };

                    let image_desc3 = Field {
                        tag: Tag::FocalLength,
                        ifd_num: In::PRIMARY,
                        value: Value::Short(vec![_exif.focal_len]),
                    };

                    let image_desc4 = Field {
                        tag: Tag::PhotographicSensitivity,
                        ifd_num: In::PRIMARY,
                        value: Value::Long(vec![_exif.iso as u32]),
                    };

                    let mut writer = Writer::new();
                    let mut buf = std::io::Cursor::new(Vec::new());
                    writer.push_field(&image_desc1);
                    writer.push_field(&image_desc2);
                    writer.push_field(&image_desc3);
                    writer.push_field(&image_desc4);
                    writer.write(&mut buf, false).unwrap();

                    let output2 = File::create(&output).unwrap();

                    let cursor = buf_writer.into_inner().unwrap();
                    let buffer = cursor.into_inner();

                    let mut jpeg = Jpeg::from_bytes(buffer.into()).unwrap();
                    jpeg.set_exif(Some(Bytes::copy_from_slice(&buf.into_inner())));
                    jpeg.encoder().write_to(output2).unwrap();
                }
                else{
                    let mut buf_writer = BufWriter::new(File::create(&output).unwrap());
                    let mut encoder =
                        jpeg_encoder::Encoder::new(&mut buf_writer, quality.try_into().unwrap());
                    encoder.set_progressive(true);
                    encoder
                        .encode(
                            &data,
                            width.try_into().unwrap(),
                            height.try_into().unwrap(),
                            jpeg_encoder::ColorType::Rgb,
                        )
                        .unwrap();
                }
            }
            
            Ok("aaa".to_string())
        }
        Some(_) => {
            let img2: image::DynamicImage = ImageBuffer::from_raw(width, height, data)
                .map(DynamicImage::ImageRgb8)
                .expect("转换错误！");
            img2.save("out.jpg").unwrap();
            Ok("aaa".to_string())
        }
        None => Err("aaa".to_string()),
    }
}

pub fn raw_process(
    input: String,
    output: String,
    lut: String,
    wb: bool,
    half_size: bool,
    exp_shift: f32,
    threshold: i32,
    quality: i32,
    embed_exif:bool,
    font_file:&str,
) -> Result<Myexif, String> {
    if let Ok(_) = fs::metadata(&input) {
        let rawdata = read_raw(&input, wb, half_size, exp_shift, threshold);
        let _exif = Myexif {
            iso: rawdata.iso,
            aperture: rawdata.aperture,
            shutter: rawdata.shutter,
            focal_len: rawdata.focal_len,
            shooting_date:rawdata.shooting_date,
        };
        if let Ok(_) = fs::metadata(&lut) {
            let lut3d = parse_cube(&lut).unwrap();
            let img = interp_8_tetrahedral(lut3d, rawdata.data, rawdata.width, rawdata.colors);
            save(
                output,
                img,
                rawdata.width.try_into().unwrap(),
                rawdata.height.try_into().unwrap(),
                quality,
                &_exif,
                embed_exif,
                font_file,
            );
            Ok(_exif)
        } else {
            save(
                output,
                rawdata.data,
                rawdata.width.try_into().unwrap(),
                rawdata.height.try_into().unwrap(),
                quality,
                &_exif,
                embed_exif,
                font_file,
            );
            Ok(_exif)
        }
    } else {
        Err("aaa".to_string())
    }
}


use std::{fs::{self, File}, io::BufWriter};
use std::ffi::{c_char, c_int, CString};
use image::{DynamicImage, ExtendedColorType, ImageBuffer, ImageEncoder};
use rayon::prelude::*;
use libraw_rs_vendor::{
    libraw_close, libraw_data_t, libraw_dcraw_make_mem_image, libraw_dcraw_process, libraw_init,
    libraw_open_file, libraw_processed_image_t, libraw_unpack, LibRaw_errors_LIBRAW_SUCCESS,
};
use webp::Encoder;
mod lut3d;
use crate::lut3d::{interp_8_tetrahedral, parse_cube};

pub struct RawData {
    data: Vec<u8>,
    width: i32,
    height: i32,
    colors: i32,
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

fn read_raw(input: String, wb: bool, half_size: bool, exp_shift: f32, threshold: i32) -> RawData {
    let libraw_data: *mut libraw_data_t = unsafe { libraw_init(0) };
    let rust_var = 0;
    let input_c_str = CString::new(input).unwrap();
    let input_c_world: *const c_char = input_c_str.as_ptr() as *const c_char;
    unsafe {
        libraw_open_file(libraw_data, input_c_world);
        libraw_unpack(libraw_data);
        (*(libraw_data)).params.exp_correc = 1;
        (*(libraw_data)).params.exp_preser = 0.8;
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
            // println!("{:?}",v);
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
        // 使用指针和大小创建一个 Rust 切片
        let _rawdata = std::slice::from_raw_parts(raw_data, raw_size as usize);

        let rawdata = RawData {
            data: _rawdata.to_vec(),
            width: _width,
            height: _height,
            colors: _colors,
        };
        libraw_close(libraw_data);
        rawdata
    }
}


fn save(output:String,data:Vec<u8>,width:u32,height:u32,quality:i32) -> Result<String,String> {
    match output.split('.').last() {
        Some(suffix) if suffix == "webp" =>{
                let encoder = Encoder::from_rgb(&data, width, height);
                let webp = encoder.encode(quality as f32);
                fs::write(output, &*webp).unwrap();
                Ok("aaa".to_string())
        }
        Some(suffix) if suffix == "jpg" => {
            let file = File::create(output).unwrap();
            let ref mut buf_writer = BufWriter::new(file);
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(buf_writer, quality.try_into().unwrap());
                encoder.write_image(
                    &data,
                    width,
                    height,
                    ExtendedColorType::Rgb8,
                ).unwrap();
                Ok("aaa".to_string())
        }
        Some(_) => {
            let img2: image::DynamicImage =
                ImageBuffer::from_raw(width, height, data)
                    .map(DynamicImage::ImageRgb8)
                    .expect("转换错误！");
            img2.save("out.jpg").unwrap();
            Ok("aaa".to_string())
        }
        None => {
            Err("aaa".to_string())
        }
    }
}

pub fn raw_process(input:String,output:String,lut:String,wb:bool,half_size:bool,exp_shift:f32,threshold:i32,quality:i32) -> Result<String,String> {
    if let Ok(_) = fs::metadata(&input){
        let rawdata = read_raw(input, wb, half_size, exp_shift, threshold);
        if let Ok(_) = fs::metadata(&lut){
            let lut3d = parse_cube(&lut).unwrap();
            let img = interp_8_tetrahedral(lut3d,rawdata.data,rawdata.width,rawdata.colors);
            save(output, img, rawdata.width.try_into().unwrap(), rawdata.height.try_into().unwrap(), quality)
        }
        else{
            save(output, rawdata.data, rawdata.width.try_into().unwrap(), rawdata.height.try_into().unwrap(), quality)
        }
    }
    else {
        Ok("aaa".to_string())
    }
    
}
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
use imageproc::drawing::{draw_filled_circle_mut, draw_filled_rect_mut};
use imageproc::rect::Rect;
use std::fs;
use std::io::Read;
use std::path::Path;

use imageproc::drawing::{draw_filled_rect, draw_line_segment_mut};
use ab_glyph::{FontArc, PxScale, Font, point};
use imageproc::drawing::draw_text_mut;

use image::{imageops::FilterType};
use imageproc::filter::gaussian_blur_f32;


struct TextMetrics {
    width: u32,
    height: u32,
}


#[derive(Debug)]
struct TextInfo {
    color: Rgba<u8>,
    font: FontArc,
    value: String,
    x: u32,
    y: u32,
}

struct IFontParam {
    size: f32,
    color: Rgba<u8>,
    font_path:String,
    case_type: Option<String>,
}

struct ITextOption {
    font: IFontParam,
    bg_height: u32,
    height: Option<u32>,
    vertical_align: String,
}


fn create_text_img(text_list: &str, opts: ITextOption) -> DynamicImage {

    // let font_data = include_bytes!(".././LXGWWenKaiMono-Regular.ttf") as &[u8];
    // let font_data = &opts.font.font_path as &[u8];
    let mut file = fs::File::open(opts.font.font_path).expect("无法打开文件");

    // 创建一个字节向量来存储文件内容
    let mut buffer = Vec::new();

    // 读取文件内容到字节向量中
    file.read_to_end(&mut buffer).expect("无法读取文件内容");

    // 将字节向量转换为字节切片
    let font_data :&'static [u8] = Box::leak(buffer.into_boxed_slice());
    
    let font = FontArc::try_from_slice(font_data).expect("Error constructing Font");
    let scale = PxScale::from(opts.font.size);

    let text_info = measure_text(&font, scale, &text_list);
    let def_text_margin = opts.bg_height as f32 * ((0.05) / 100.0); // Example margin

    let height = opts.height.unwrap_or_else(|| {
        (text_info.height as f32 + def_text_margin * 2.0).ceil() as u32
    });

    let mut img = DynamicImage::new_rgba8(text_info.width, height); // Initial width

    draw_text_mut(&mut img, opts.font.color, 0,0, scale, &font, text_list);

    img
}

fn measure_text(font: &FontArc, scale: PxScale, text: &str) -> TextMetrics {
    let mut width = 0.0;
    let mut height:f32 = 0.0;

    for c in text.chars() {
        let glyph = font.glyph_id(c).with_scale(scale);
        let _bounding_box = font.glyph_bounds(&glyph);
        width += _bounding_box.width();
        height = height.max(_bounding_box.height());
        // println!("{} {:?}",c,_bounding_box.width());
        
    }
    // println!("{}",width as u32);

    TextMetrics {
        width: width as u32,
        height: height as u32,
    }
}


pub fn gen_frame_img(main_img:Vec<u8>,width:u32,height:u32,text_list: &str,solid_bg:bool,shadow_show:bool,shadow:Option<f32>,font_path:&str) -> (Vec<u8>,u32,u32){

    let main_img = DynamicImage::ImageRgb8(ImageBuffer::from_raw(width,height,main_img).unwrap());
    main_img.save("aaabb.jpg");

    let (bg_h,bg_w) = calc_bg_img_size(main_img.height(), main_img.width(), main_img.height(), main_img.width(), 100.0, Some(main_img.height()));

    // println!("{} {}",bg_h,bg_w);

    

    let (content_h,m_top) = calc_content_height(bg_h,main_img.height(),Some(5.0),0,true,Some(50));

    let mut _content_offset_y = m_top;

    let (bg_h,bg_w) = calc_bg_img_size(main_img.height(), main_img.width(), main_img.height(), main_img.width(), 100.0, Some(content_h.try_into().unwrap()));

    // println!("{} {}",bg_h,bg_w);

    let opts = ITextOption {
        font: IFontParam {
            size: 48.0,
            color: Rgba([255, 255, 255, 255]),
            font_path:font_path.to_string(),
            case_type: None,
        },
        bg_height: bg_h,
        height: Some(50),
        vertical_align: "center".to_string(),
    };

    let text_img = create_text_img(text_list,opts);

    

    let _bg_img = main_img.resize(bg_w, bg_h, FilterType::Nearest).to_rgba8();
    let mut bg_img = gaussian_blur_f32(&_bg_img, 200.0);

    let content_offset_x = (bg_w - main_img.width()) /2;
    _content_offset_y += ((bg_h - content_h as u32) /2) as i32;
    let content_offset_y = _content_offset_y as u32;

    let text_offset_x = (bg_w - text_img.width()) /2;
    let text_offset_y =  content_offset_y + main_img.height() + (bg_h - content_offset_y - main_img.height()) /2 - (text_img.height() /2 );

    // println!("{} {}",text_offset_x,text_offset_y);

    // Add black overlay
    if !solid_bg {
        let average_brightness = calc_average_brightness(&bg_img);
        // println!("{}",average_brightness);

        let overlay_color = if average_brightness < 15 {
            Rgba([180, 180, 180, 51]) // rgba(180, 180, 180, 0.2)
        } else if average_brightness < 20 {
            Rgba([158, 158, 158, 51]) // rgba(158, 158, 158, 0.2)
        } else if average_brightness < 40 {
            Rgba([128, 128, 128, 51]) // rgba(128, 128, 128, 0.2)
        } else {
            Rgba([0, 0, 0, 51]) // rgba(0, 0, 0, 0.2)
        };

        let mut _canvas = DynamicImage::new_rgba8(bg_w, bg_h).to_rgba8();

        draw_filled_rect_mut(
            &mut _canvas,
            Rect::at(0, 0).of_size(bg_w,bg_h),
            overlay_color,
        );
        image::imageops::overlay(&mut bg_img, &_canvas, 0, 0);

    }

    // let corner_radius = 100; // 假设圆角半径为10

    // 创建圆角矩形蒙版
    // let mut mask = ImageBuffer::new(main_img.width(), main_img.height());
    // draw_rounded_rect(&mut mask, Rect::at(0, 0).of_size(main_img.width(), main_img.height()), blur as u32, Rgba([255, 255, 255, 255]));

    // let rounded_main_img = apply_mask(&main_img, &mask);


    // 添加阴影
    let shadow_blur = 20.0; // 假设阴影模糊半径为5
    let shadow_offset = 10; // 假设阴影偏移量为5

    let mut shadow = ImageBuffer::new(bg_w,bg_h);
    draw_filled_rect_mut(&mut shadow, Rect::at((content_offset_x - shadow_offset) as i32,(content_offset_y - shadow_offset) as i32).of_size(main_img.width() + (2* shadow_offset) as u32,main_img.height() + (2* shadow_offset)), Rgba([0,0,0,128]));
    let shadowa = gaussian_blur_f32(&shadow, shadow_blur);

    image::imageops::overlay(&mut bg_img, &shadowa, 0,0);

    // 绘制主体图片到画布
    image::imageops::overlay(&mut bg_img, &main_img, content_offset_x as i64, content_offset_y as i64);

    image::imageops::overlay(&mut bg_img, &text_img, text_offset_x as i64, text_offset_y as i64);

    let out = DynamicImage::ImageRgba8(bg_img).to_rgb8();
    (out.to_vec(),out.width(),out.height())

}

fn calc_average_brightness(img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> u8 {
    let mut sum = 0;
    let mut count = 0;
    for pixel in img.pixels() {
        sum += (pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32) / 3;
        count += 1;
    }
    (sum / count) as u8
}

fn calc_bg_img_size(reset_h:u32,reset_w:u32,h:u32,w:u32,main_img_w_rate:f32, height: Option<u32>) ->(u32,u32) {

    let mut reset_height = reset_h;
    let mut reset_width = reset_w;

    let wh_rate = reset_width as f32 / reset_height as f32;

    if let Some(h) = height {
        reset_height = h;
        reset_width = (reset_height as f32 * wh_rate).ceil() as u32;
    } else {
        let valid_height = if h > reset_height { h } else { reset_height };
        reset_height = valid_height;
        reset_width = (reset_height as f32 * wh_rate).ceil() as u32;
    }

    let main_img_width_rate = main_img_w_rate / 100.0;
    if w as f32 / reset_width as f32 > main_img_width_rate {
        reset_width = (w as f32 / main_img_width_rate).ceil() as u32;
        reset_height = (reset_width as f32 / wh_rate).ceil() as u32;
    }

    // self.material.bg = Some(ImageInfo {
    //     path: self.output_file_names.bg.clone(),
    //     h: reset_height,
    //     w: reset_width,
    //     top: 0,
    //     left: 0,
    // });
    (reset_height,reset_width)
}

fn calc_content_height(bg_h:u32,m_h:u32,shadow:Option<f32>,mini_top_bottom_margin:i32,shadow_show:bool,text:Option<u32>) -> (i32,i32) {
    // let opt = &self.output_opt;
    let bg_height = bg_h;
    let main_img_top_offset = bg_height as f32 * (mini_top_bottom_margin as f32 / 100.0);
    let text_bottom_offset = bg_height as f32 * 0.027;

    // 主图上下间隔最小间隔
    let mut content_top = main_img_top_offset.ceil();
    let mut main_img_offset = content_top * 2.0;

    // 阴影宽度
    if shadow_show {
        let shadow_height = (m_h as f32 * (shadow.unwrap_or(0.0) / 100.0)).ceil();
        content_top = content_top.max(shadow_height);
        main_img_offset = content_top * 2.0;
    }

    // 有文字时文字与主图的间隔要小于主图对顶部的间隔，并且底部间隔使用文字对底部的间隔
    if let _ = Some(text) {
        main_img_offset *= 0.75;
        main_img_offset += text_bottom_offset;
    }

    // 文本高度
    // let text_h = text.iter().map(|i| i.0).sum::<i32>();

    // 生成背景图片
    let content_h = text.unwrap_or(0) as i32 + m_h as i32 + main_img_offset.ceil() as i32;

    let m_top = content_top as i32;
    // let content_h = content_h;

    (content_h,m_top)
}

fn draw_rounded_rect(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, rect: Rect, radius: u32, color: Rgba<u8>) {
    // let Rect { x: x0, y: y0, width: w, height: h } = rect;
    let x0 = rect.left();
    let y0 = rect.top();
    let w = rect.width();
    let h = rect.height();

    let x1 = x0 + w as i32;
    let y1 = y0 + h as i32;
    let r = radius as i32;

    // 绘制矩形主体
    draw_filled_rect_mut(img, Rect::at(x0 + r, y0).of_size(w - 2 * r as u32, h), color);
    draw_filled_rect_mut(img, Rect::at(x0, y0 + r).of_size(r as u32, h - 2 * r as u32), color);
    draw_filled_rect_mut(img, Rect::at(x1 - r, y0 + r).of_size(r as u32, h - 2 * r as u32), color);
    draw_filled_rect_mut(img, Rect::at(x0 + r, y1 - r).of_size(w - 2 * r as u32, r as u32), color);

    // 绘制圆角
    draw_filled_circle_mut(img, (x0 + r, y0 + r), r as i32, color);
    draw_filled_circle_mut(img, (x1 - r, y0 + r), r as i32, color);
    draw_filled_circle_mut(img, (x0 + r, y1 - r), r as i32, color);
    draw_filled_circle_mut(img, (x1 - r, y1 - r), r as i32, color);
}

fn apply_mask(img: &DynamicImage, mask: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut result = img.to_rgba8();
    for (x, y, pixel) in result.enumerate_pixels_mut() {
        let mask_pixel = mask.get_pixel(x, y);
        if mask_pixel[3] == 0 {
            *pixel = Rgba([0, 0, 0, 0]); // 透明
        }
    }
    result
}

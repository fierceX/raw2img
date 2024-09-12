use raw::raw_process;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// 输入文件路径
    #[arg(short, long)]
    input: String,

    /// 输出文件路径
    #[arg(short, long)]
    output: String,

    /// 使用 lut 文件滤镜
    #[arg(short, long, default_value = "")]
    lut: String,

    /// 使用自动白平衡或相机白平衡
    #[arg(short, long)]
    auto_wb: bool,

    /// 输出尺寸减半
    #[arg(short, long,default_value_t = false)]
    half_size: bool,

    /// 曝光偏移，值范围为 0.25-8，从降低两档到提升三档。当该值指定时，自动曝光偏移将不起作用
    #[arg(short, long,default_value_t = -3.0)]
    exp_shift: f32,

    /// 输出质量，值范围 0-100
    #[arg(short, long,default_value_t = 90)]
    quality: i32,

    /// 降噪参数。当指定该值时，自动降噪将不起作用
    #[arg(short, long,default_value_t = -1)]
    noise: i32,

    /// 是否嵌入exif
    #[arg(short, long ,default_value_t = true)]
    embed_exif: bool,

    /// 边框字体，当指定该值时，则会添加边框
    #[arg(short, long ,default_value = "")]
    font_file: String,
}


fn main() {
    let args = Args::parse();
    let _ = raw_process(args.input,args.output,args.lut, args.auto_wb, args.half_size, args.exp_shift, args.noise,args.quality,args.embed_exif,&args.font_file);
}

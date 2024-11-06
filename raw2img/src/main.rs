use std::fmt::format;

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{cookie::Key, middleware::{Logger,Compress}, web::Data, App, HttpServer};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web_static_files::ResourceFiles;

use clap::{Args, Command,Subcommand, Parser};
use lazy_static::lazy_static;
use raw::raw_process;


mod db;
mod handlers;
mod schemas;
mod proces;

use self::{db::{create_tantivy_index,sync_sqlite_to_tantivy,get_db_pool}, handlers::register};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));
include!(concat!(env!("OUT_DIR"), "/git_info.rs"));


#[derive(Parser)]
#[command(about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}


#[derive(Subcommand)]
enum Commands {

    /// 启动Web服务
    Server(ServerArgs),

    /// 转换Raw文件
    Convert(ConvertArgs),
}

#[derive(Args)]
struct ServerArgs {
    /// 绑定IP地址
    #[arg(short, long, default_value = "0.0.0.0")]
    bind: String,

    /// 监听端口号
    #[arg(short, long, default_value = "8081")]
    port: i32,

    /// 数据库路径
    #[arg(short, long, default_value = "db.db")]
    database: String,

    /// 索引路径
    #[arg(short, long, default_value = "tantivy_index")]
    index: String,
}

#[derive(Args)]
struct ConvertArgs {
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

lazy_static! {
    static ref VERSION: String = format!("{} (build {})", env!("CARGO_PKG_VERSION"), GIT_HASH);
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let matches = Command::new("Raw2Img")
        .version(VERSION.as_str());


    let cli = Cli::augment_args(matches);
    let matches = cli.get_matches();


    match matches.subcommand() {
        Some(("server", sub_matches)) => 
            {
                let bind = sub_matches.get_one::<String>("bind").unwrap();
                let port = sub_matches.get_one::<i32>("port").unwrap();
                let database = sub_matches.get_one::<String>("database").unwrap();
                let index_path = sub_matches.get_one::<String>("index").unwrap();
                let bindaddr = format!("{}:{}",bind,port);

                env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

                let pool = get_db_pool(database);
                let index = create_tantivy_index(index_path).unwrap();

                sync_sqlite_to_tantivy(&pool,&index);

                log::info!("启动服务：http://{}",bindaddr);

                std::fs::create_dir_all("./tmp")?;

                HttpServer::new(move || {
                    let generated = generate();
                    App::new()
                        
                        .app_data(Data::new(pool.clone()))
                        .app_data(Data::new(index.clone()))
                        .configure(register)
                        .wrap(Cors::permissive())
                        .service(Files::new("/tmp", "./tmp"))
                        .service(ResourceFiles::new("/", generated))
                        .wrap(Compress::default())
                        .wrap(
                            SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                                .cookie_secure(false)
                                .build(),
                        )
                        .wrap(Logger::default())
                })
                .workers(2)
                .bind(bindaddr)?
                .run()
                .await
            },
        Some(("convert",sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            let output = sub_matches.get_one::<String>("output").unwrap();
            let lut = sub_matches.get_one::<String>("lut").unwrap();
            let auto_wb = sub_matches.get_one::<bool>("auto_wb").unwrap();
            let half_size = sub_matches.get_one::<bool>("half_size").unwrap();
            let exp_shift = sub_matches.get_one::<f32>("exp_shift").unwrap();
            let noise = sub_matches.get_one::<i32>("noise").unwrap();
            let quality = sub_matches.get_one::<i32>("quality").unwrap();
            let embed_exif = sub_matches.get_one::<bool>("embed_exif").unwrap();
            let font_file = sub_matches.get_one::<String>("font_file").unwrap();

            println!("{}",font_file);

            let _ = raw_process(input.clone(),output.clone(),lut.clone(), *auto_wb, *half_size, *exp_shift, *noise,*quality,*embed_exif,font_file);
            Ok(())
        },
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
    
}

use std::process::Command;

use actix_cors::Cors;
use actix_files::Files;
use actix_multipart::{
    form::{
        tempfile::{TempFile, TempFileConfig},
        MultipartForm,
    },
    Multipart,
};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{cookie::Key, middleware, web, App, Error, HttpResponse, HttpServer, Responder};
use actix_web_static_files::ResourceFiles;
use base16ct;
use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use blake2;
use serde::{Deserialize, Serialize};
use std::fs;
use chrono::{DateTime, Utc};

use raw::raw_process;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file")]
    files: Vec<TempFile>,
}

#[derive(Deserialize, Debug)]
struct Parameters {
    filename: String,
    lut: String,
    wb: bool,
    exp_shift: f64,
    threshold: i32,
}

async fn raw2jpg(session: Session, parames: web::Json<Parameters>) -> HttpResponse {
    let mut dir_path = String::new();
    if let Ok(Some(userid)) = session.get::<String>("userid") {
        // filename = format!("{}", _count);
        dir_path = format!("./tmp/{}",userid);
        let intput_file_path = format!("{}/{}", dir_path,parames.filename);
        log::info!("{}",intput_file_path);
     
    if let Ok(_) = fs::metadata(intput_file_path.clone()) {
        let mut hasher = Blake2bVar::new(10).unwrap();

        let mut buf = [0u8; 10];
        hasher.update(
            format!(
                "{}{}{}{}{}",
                parames.filename, parames.lut, parames.wb, parames.exp_shift, parames.threshold
            )
            .as_bytes(),
        );
        hasher.finalize_variable(&mut buf).unwrap();
        // log::info!("{}", out_file);
        let out_file_path = format!("{}/{}.webp",dir_path, base16ct::lower::encode_string(&buf));
        log::info!("{}",out_file_path);
        if let Ok(_) = fs::metadata(out_file_path.clone()) {
            HttpResponse::Ok().json(out_file_path)
        } else {
            log::info!("{}: {:?}",userid, parames);
            let _ = raw_process(intput_file_path,out_file_path.clone(),format!("./lut/{}.cube", parames.lut),parames.wb,false,parames.exp_shift as f32,parames.threshold,90);
            HttpResponse::Ok().json(out_file_path)
        }
    } else {
        log::info!("ccc");
        HttpResponse::NotFound().finish()
    }
}
    else{
        log::info!("bbbaa");
        HttpResponse::NotFound().finish()
    }
}

async fn save_files(
    session: Session,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> Result<impl Responder, Error> {
    let mut userid = String::new();

    if let Ok(Some(_count)) = session.get::<String>("userid") {
        
        // userid = timestamp;
        userid = _count;

    } else {
        let now: DateTime<Utc> = Utc::now();
    
        // 将时间格式化为时间戳字符串
        userid = now.format("%s").to_string();
        std::fs::create_dir_all(format!("./tmp/{}",userid))?;

        let _ = session.insert("userid", userid.clone());
    }

    let save_path = format!("./tmp/{}/",userid);

    for f in form.files {
        let path = format!("{}/{}", save_path,f.file_name.as_ref().unwrap());
        log::info!("saving to {path}");
        f.file.persist(path).unwrap();
    }

    Ok(HttpResponse::Ok())
}

async fn find_lut() -> HttpResponse {
    let mut luts: Vec<String> = Vec::new();
    for entry in fs::read_dir("./lut").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        // let metadata = fs::metadata(&path).unwrap();
        let pathp = path.as_path();
        if pathp.extension().unwrap() == "cube" {
            let lut_file = pathp.file_stem().unwrap();
            luts.push(format!("{}", lut_file.to_str().unwrap()));
        }
    }
    HttpResponse::Ok().json(luts)
}

async fn get_rawfiles(session: Session) -> HttpResponse {
    let mut rawfiles: Vec<String> = Vec::new();
    let mut dir_path = String::new();
    if let Ok(Some(userid)) = session.get::<String>("userid") {
        // filename = format!("{}", _count);
        dir_path = format!("./tmp/{}",userid);
        if !std::path::Path::new(&dir_path).is_dir(){
            let _ = std::fs::create_dir_all(&dir_path);
        }
        
        // let intput_file_path = format!("{}/{}", dir_path,parames.filename);

        for entry in fs::read_dir(dir_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            // let metadata = fs::metadata(&path).unwrap();
            let pathp = path.as_path();
            if pathp.extension().unwrap() != "jpg" {
                let raw_file = pathp.file_name().unwrap();
                rawfiles.push(format!("{}", raw_file.to_str().unwrap()));
            }
    }
}
    HttpResponse::Ok().json(rawfiles)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // log::info!("creating temporary upload directory");
    std::fs::create_dir_all("./tmp")?;

    // log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(|| {
        let generated = generate();
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(false)
                    .build(),
            )
            .wrap(
                Cors::permissive(),
            )
            .app_data(TempFileConfig::default().directory("./tmp"))
            .service(Files::new("/tmp", "./tmp/"))
            .service(web::resource("/upfile").route(web::post().to(save_files)))
            .service(web::resource("/rawfiles").route(web::get().to(get_rawfiles)))
            .service(web::resource("/raw2jpg").route(web::post().to(raw2jpg)))
            .service(web::resource("/luts").route(web::get().to(find_lut)))
            .service(ResourceFiles::new("/", generated))
    })
    .bind(("0.0.0.0", 8081))?
    .workers(2)
    .run()
    .await
}

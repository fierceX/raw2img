use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_multipart::{
    form::{
        tempfile::{TempFile, TempFileConfig},
        MultipartForm,
    },
    Multipart,
};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    cookie::Key,
    middleware,
    web::{self, BufMut},
    App, Error, HttpResponse, HttpServer, Responder,
};
use actix_web_static_files::ResourceFiles;

use blake2;
use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use chrono::{DateTime, Utc};
use exif::{In, Tag, Value};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File}, path::Path,
};

use raw::raw_process;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file")]
    files: Vec<TempFile>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Parameters {
    filename: String,
    lut: String,
    wb: bool,
    exp_shift: f64,
    threshold: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Myexif {
    iso: i32,
    aperture: f32,
    shutter: f32,
    focal_len: i32,
    filename: String,
    url: String,
}

async fn raw2jpg(session: Session, parames: web::Json<Parameters>) -> HttpResponse {
    let mut dir_path = String::new();
    if let Ok(Some(userid)) = session.get::<String>("userid") {
        dir_path = format!("./tmp/{}", userid);
        let intput_file_path = format!("{}/{}", dir_path, parames.filename);

        if let Ok(_) = fs::metadata(intput_file_path.clone()) {
            let mut hasher = Blake2bVar::new(10).unwrap();

            let mut buf = [0u8; 10];
            hasher.update(
                format!(
                    "{}{}{}{}{}{}",
                    userid,
                    parames.filename,
                    parames.exp_shift,
                    parames.lut,
                    parames.threshold,
                    parames.wb
                )
                .as_bytes(),
            );
            hasher.finalize_variable(&mut buf).unwrap();

            let _ = std::fs::create_dir_all(format!("{}/tmp/", dir_path));
            let out_file_name = format!("tmp/{}.jpg", base16ct::lower::encode_string(&buf));
            let out_file_path = format!(
                "{}/{}",
                dir_path,
                out_file_name
            );

            if let Ok(_) = fs::metadata(out_file_path.clone()) {
                HttpResponse::Ok().json(format!("/img/{}",out_file_name))
            } else {
                let _ = raw_process(
                    intput_file_path,
                    out_file_path.clone(),
                    format!("./lut/{}.cube", parames.lut),
                    parames.wb,
                    true,
                    parames.exp_shift as f32,
                    parames.threshold,
                    90,
                );
                HttpResponse::Ok().json(format!("/img/{}",out_file_name))
            }
        } else {
            HttpResponse::NotFound().finish()
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}

async fn savejpg(session: Session, parames: web::Json<(String, String)>) -> HttpResponse {
    if let Ok(Some(userid)) = session.get::<String>("userid") {
        let _parames = parames.0;
        let url_file = format!("./tmp/{}/tmp/{}", userid, _parames.0);
        let file_name = format!(
            "./tmp/{}/{}.jpg",
            userid,
            _parames.1.split(".").next().unwrap()
        );
        fs::rename(url_file, file_name);
        HttpResponse::Ok().into()
    } else {
        HttpResponse::Ok().into()
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
        std::fs::create_dir_all(format!("./tmp/{}", userid))?;

        let _ = session.insert("userid", userid.clone());
    }

    let save_path = format!("./tmp/{}/", userid);

    for f in form.files {
        let path = format!("{}/{}", save_path, f.file_name.as_ref().unwrap());
        f.file.persist(path).unwrap();
    }

    Ok(HttpResponse::Ok())
}

async fn find_lut() -> HttpResponse {
    let mut luts: Vec<String> = Vec::new();
    for entry in fs::read_dir("./lut").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
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
    let mut jpgs = HashMap::new();
    let mut dir_path = String::new();
    let mut res = Vec::new();
    if let Ok(Some(userid)) = session.get::<String>("userid") {
        dir_path = format!("./tmp/{}", userid);
        if !std::path::Path::new(&dir_path).is_dir() {
            let _ = std::fs::create_dir_all(&dir_path);
        }

        for entry in fs::read_dir(dir_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path().clone();
            let patha = path.clone();
            let pathp: &std::path::Path = path.as_path();
            if pathp.is_file() {
                if pathp.extension().unwrap() == "jpg" {
                    let file_name = pathp.file_stem().unwrap().to_str().unwrap();

                    let file = std::fs::File::open(&patha).unwrap();
                    let mut bufreader = std::io::BufReader::new(&file);
                    let exifreader = exif::Reader::new();
                    let exif = exifreader.read_from_container(&mut bufreader).unwrap();

                    let mut aperture = 0.0;
                    let mut iso = 0;
                    let mut shutter = 0.0;
                    let mut focal_len = 0;

                    if let Value::Float(val) =
                        &exif.get_field(Tag::FNumber, In::PRIMARY).unwrap().value
                    {
                        aperture = val[0]
                    } else {
                        println!("Not an F32 value");
                    }

                    if let Value::Long(val) = &exif
                        .get_field(Tag::PhotographicSensitivity, In::PRIMARY)
                        .unwrap()
                        .value
                    {
                        iso = val[0]
                    } else {
                        println!("Not an F32 value");
                    }

                    if let Value::Float(val) = &exif
                        .get_field(Tag::ExposureTime, In::PRIMARY)
                        .unwrap()
                        .value
                    {
                        shutter = val[0];
                    } else {
                        println!("Not an F32 value");
                    }

                    if let Value::Short(val) =
                        &exif.get_field(Tag::FocalLength, In::PRIMARY).unwrap().value
                    {
                        focal_len = val[0];
                    } else {
                        println!("Not an F32 value");
                    }

                    let mut _exif = Myexif {
                        aperture: aperture,
                        iso: iso.try_into().unwrap(),
                        shutter: shutter,
                        focal_len: focal_len.into(),
                        filename: file_name.to_owned(),
                        url: format!("/img/{}.jpg", file_name),
                    };
                    jpgs.insert(_exif.filename.clone(), _exif);
                } else {
                    let raw_file = pathp.file_name().unwrap();
                    rawfiles.push(format!("{}", raw_file.to_str().unwrap()));
                }
            }
        }

        for k in rawfiles.iter() {
            let kk = k.split(".").next().unwrap();
            if jpgs.contains_key(kk) {
                let mut _j = jpgs.remove(kk).unwrap();
                _j.filename = k.to_string();
                res.push(_j);
            } else {
                res.push(Myexif {
                    filename: k.to_string(),
                    iso: 0,
                    aperture: 0.0,
                    shutter: 0.0,
                    focal_len: 0,
                    url: "".to_string(),
                });
            }
        }
    }
    HttpResponse::Ok().json(res)
}

async fn get_image(session: Session,url:web::Path<String>) -> Result<impl Responder,Error>{
    log::info!("bbffdd");
    if let Ok(Some(userid)) = session.get::<String>("userid") {
        
        let path = format!("tmp/{}/{}", userid,url);
        log::info!("{}",path);
        if Path::new(&path).exists() {
            Ok(NamedFile::open(path).unwrap())
        } else {
            Err(actix_web::error::ErrorNotFound("Image not found"))
        }
    }
    else {
        Err(actix_web::error::ErrorNotFound("Image not found"))
    }
    
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
            .wrap(Cors::permissive())
            .app_data(TempFileConfig::default().directory("./tmp"))
            .service(web::resource("/upfile").route(web::post().to(save_files)))
            .service(web::resource("/rawfiles").route(web::get().to(get_rawfiles)))
            .service(web::resource("/raw2jpg").route(web::post().to(raw2jpg)))
            .service(web::resource("/luts").route(web::get().to(find_lut)))
            .service(web::resource("/save").route(web::post().to(savejpg)))
            .service(web::resource("/img/{path:.*}").route(web::get().to(get_image)))
            .service(ResourceFiles::new("/", generated))
    })
    .bind(("0.0.0.0", 8081))?
    .workers(2)
    .run()
    .await
}

use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_multipart::{
    form::{
        tempfile::{TempFile, TempFileConfig},
        MultipartForm,
    },
    Multipart,
};
// use actix_web::middleware::{Middleware, Started};
use actix_web::{body::MessageBody, dev::{Response, ServiceRequest, ServiceResponse}, middleware::{self, Next}, route, ResponseError};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    cookie::Key,
    web::{self, BufMut},
    App, Error, HttpResponse, HttpServer, Responder,
};
use actix_web_static_files::ResourceFiles;

use blake2;
use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use chrono::{DateTime, Utc};
use exif::{In, Tag, Value};
use jwt_simple::{claims::{Claims, NoCustomClaims}, prelude::{Duration, HS256Key, MACLike}};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap, fmt, fs::{self, File}, path::Path
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


#[derive(Debug, Serialize, Deserialize)]
struct FormData {
    username: String,
    password: String,
}

// #[derive(Debug)]
// pub enum AuthenticationError {
//     InternalServerError,
//     AuthRequest(String),
// }

// impl fmt::Display for AuthenticationError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "aaa")
//     }
// }

// impl ResponseError for AuthenticationError {
//     fn error_response(&self) -> HttpResponse {
//         match *self {
//             AuthenticationError::InternalServerError => HttpResponse::SeeOther()
//                 .append_header(("Location", "/login.html"))
//                 .finish(),
//             AuthenticationError::AuthRequest(ref message) => HttpResponse::SeeOther()
//                 .append_header(("Location", "/login.html"))
//                 .finish(),
//         }
//     }
    
//     fn status_code(&self) -> actix_web::http::StatusCode {
//         actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
//     }
// }


async fn authentication(
    session: Session,
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    // pre-processing
    log::info!("vvddd");
    if let (Ok(Some(userid)),Ok(Some(passkey))) = (session.get::<String>("userid"),session.get::<String>("passkey")) {
        log::info!("v: {} {}",userid,passkey);
        let key = HS256Key::from_bytes(&"key".try_into_bytes().unwrap());
        match key.verify_token::<NoCustomClaims>(&passkey, None){
            Ok(_claims) => {
                let res = next.call(req).await?;
                Ok(res)
            }
            Err(_) =>{
                Err(actix_web::error::ErrorUnauthorized("no auth"))
            }
        }
        
    }
    else{
        // Err(AuthenticationError::AuthRequest("Invalid input".to_string()).into())
        Err(actix_web::error::ErrorUnauthorized("no auth"))
    }
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
    if let Ok(Some(userid)) = session.get::<String>("userid") {
        let save_path = format!("./tmp/{}/", userid);

        for f in form.files {
            let path = format!("{}/{}", save_path, f.file_name.as_ref().unwrap());
            f.file.persist(path).unwrap();
        }

        Ok(HttpResponse::Ok())
    } else {
        Err(actix_web::error::ErrorUnauthorized("aaa"))
    }

    
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
        log::info!("vvaa {}",userid);
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
                        url: format!("/api/img/{}.jpg", file_name),
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

async fn auth(session: Session,form: web::Form<FormData>) -> HttpResponse{
    if let Ok(Some(userid)) = session.get::<String>("userid") {
        log::info!("aaa: {}",userid);
    }
    else{
        let _r = session.insert("userid", form.username.clone());
    }
    if let Ok(Some(userid)) = session.get::<String>("passkey") {
        log::info!("bbb: {}",userid);
    }
    else{
        let key = HS256Key::from_bytes(&"key".try_into_bytes().unwrap());
        let claims = Claims::create(Duration::from_hours(2));
        let token = key.authenticate(claims).unwrap();
        log::info!("token: {}",token);
        let _r = session.insert("passkey", token);
    }
    HttpResponse::SeeOther()
                .append_header(("Location", "/"))
                .finish()
}

async fn check_auth(session: Session) -> HttpResponse{
    HttpResponse::Ok().into()
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
            .wrap(Cors::permissive())
            .service(web::scope("/api")
            .wrap(middleware::from_fn(authentication))
            .service(web::resource("/upfile").route(web::post().to(save_files)))
            .service(web::resource("/rawfiles").route(web::get().to(get_rawfiles)))
            .service(web::resource("/raw2jpg").route(web::post().to(raw2jpg)))
            .service(web::resource("/save").route(web::post().to(savejpg)))
            .service(web::resource("/img/{path:.*}").route(web::get().to(get_image)))
            .service(web::resource("/luts").route(web::get().to(find_lut)))
            .service(web::resource("/check_auth").route(web::post().to(check_auth)))
        )
        
        .app_data(TempFileConfig::default().directory("./tmp"))
        .service(web::resource("/auth").route(web::post().to(auth)))
        .service(ResourceFiles::new("/", generated))
        .wrap(
            SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                .cookie_secure(false)
                .build(),
        )
        .wrap(middleware::Logger::default())
    })
    .bind(("0.0.0.0", 8081))?
    .workers(2)
    .run()
    .await
}

use actix_files::NamedFile;
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    body::MessageBody,
    dev::{Response, ServiceRequest, ServiceResponse},
    middleware::{self, Next},
};
use actix_web::{cookie::Key, get, route, web, Error, HttpResponse, Responder};
use blake2::{Blake2b512, Blake2s256, Digest};
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};
use jwt_simple::{
    claims::{Claims, NoCustomClaims},
    prelude::{Duration, HS256Key, MACLike},
};
use raw::{raw_process,add_frame};
use rusqlite::named_params;
use serde::{Deserialize, Serialize};
use tantivy::Index;

use std::sync::{Arc, Mutex};
use std::{
    fs,
    path::{self, Path},
    thread,
};

use crate::{
    db::{create_tantivy_index, get_db_pool, sync_sqlite_to_tantivy, Pool},
    proces::{self, raw2img, scan_files},
    schemas::{
        root::{create_schema, Context, Schema},
        storage,
    },
};


#[derive(Deserialize,Debug)]
struct PhoframeQuery {
    phoframe: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FormData {
    username: String,
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FormData2 {
    username: String,
    password: String,
}

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file")]
    files: Vec<TempFile>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Parameters {
    pub id: i32,
    pub filename: String,
    pub lut: String,
    pub wb: bool,
    pub exp_shift: f64,
    pub threshold: i32,
}

/// GraphQL endpoint
#[route("/graphql", method = "GET", method = "POST")]
pub async fn graphql(
    index: web::Data<Index>,
    pool: web::Data<Pool>,
    schema: web::Data<Schema>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let ctx = Context {
        db_pool: pool.get_ref().to_owned(),
        index:index.get_ref().to_owned(),
    };

    let res = data.execute(&schema, &ctx).await;

    Ok(HttpResponse::Ok().json(res))
}

/// GraphiQL UI
#[get("/graphiql")]
async fn graphql_playground() -> impl Responder {
    web::Html::new(graphiql_source("/api/graphql", None))
}

#[route("/check_auth", method = "POST")]
async fn check_auth(session: Session) -> HttpResponse {
    // HttpResponse::Ok().into()
    if let (Ok(Some(userid)), Ok(Some(passkey))) = (
        session.get::<String>("userid"),
        session.get::<String>("passkey"),
    ) {
        // log::info!("v: {} {}", userid, passkey);
        let key = HS256Key::from_bytes(&"key".try_into_bytes().unwrap());
        match key.verify_token::<NoCustomClaims>(&passkey, None) {
            Ok(_claims) => {
                // let res = next.call(req).await?;
                // Ok(res)
                let user_id = _claims.audiences.unwrap().into_string().unwrap();
                HttpResponse::Ok().json(user_id)
            }
            Err(_) => HttpResponse::Unauthorized().body(""),
        }
    } else {
        HttpResponse::Unauthorized().body("")
    }
    // HttpResponse::Ok().json("1".to_string())
}

#[route("/create_user", method = "POST")]
async fn create_user(
    session: Session,
    pool: web::Data<Pool>,
    form: web::Form<FormData>,
) -> HttpResponse {
    let db_conn = pool.get_ref().to_owned();

    // let (user_id,password):(i32,String) = db_conn.get().unwrap().query_row("select id,password from users where username = :username;", named_params!{":id":&form.username}, |row| (row.get(0),row.get(1)),).unwrap();
    let mut hasher = Blake2s256::new();
    // let mut buf = [0u8; 256];
    hasher.update(form.password.as_bytes());
    // hasher.finalize_variable(&mut buf).unwrap();
    let buf = hasher.finalize();
    let input_password = base16ct::lower::encode_string(&buf);
    let res = db_conn.get().unwrap().execute(
        "INSERT INTO users (username, email, password, wb, half_size, quality) VALUES (?1, ?2, ?3, false, true, 90)",
        (&form.username, &form.email, &input_password),
    );
    match res {
        Ok(_) => HttpResponse::Ok().body(""),
        Err(_) => HttpResponse::Unauthorized().body(""),
    }
}

#[route("/auth", method = "POST")]
async fn auth(session: Session, pool: web::Data<Pool>, form: web::Form<FormData2>) -> HttpResponse {
    // log::info!("bbddeevv123");
    let db_conn = pool.get_ref().to_owned();

    let (user_id, password): (i32, String) = db_conn
        .get()
        .unwrap()
        .query_row(
            "select id,password from users where username = :username;",
            named_params! {":username":&form.username},
            |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())),
        )
        .unwrap();
    let mut hasher = Blake2s256::new();
    // let mut buf = [0u8; 256];
    hasher.update(form.password.as_bytes());
    let buf = hasher.finalize();
    let input_password = base16ct::lower::encode_string(&buf);

    if password == input_password {
        let key = HS256Key::from_bytes(&"key".try_into_bytes().unwrap());
        let claims = Claims::create(Duration::from_hours(2)).with_audience(user_id.to_string());
        let token = key.authenticate(claims).unwrap();
        log::info!("token: {}", token);
        let _r = session.insert("passkey", token);
        let _r = session.insert("userid", user_id.to_string());

        // HttpResponse::SeeOther()
        //             .append_header(("Location", "/"))
        //             .finish()
        HttpResponse::Ok().json(user_id)
    } else {
        HttpResponse::Unauthorized().body("")
    }
}

#[route("/scan", method = "POST")]
async fn scans(pool: web::Data<Pool>,index: web::Data<Index>, user_id: web::Json<i32>) -> HttpResponse {
    let _pool = Arc::new(Mutex::new(pool.get_ref().to_owned()));
    let _index = index.get_ref().to_owned();
    let _handle = thread::spawn(move || {
        let _pool1 = _pool.clone();
        scan_files(*user_id, _pool1);
        let _pool2 = _pool.lock().unwrap();
        sync_sqlite_to_tantivy(&_pool2, &_index);
    });
    HttpResponse::Ok().body("")
}

#[route("/img/{path:.*}", method = "GET")]
async fn get_image(
    session: Session,
    info: web::Query<PhoframeQuery>,
    pool: web::Data<Pool>,
    url: web::Path<String>,
    
) -> Result<impl Responder, Error> {
    let (storage_name, _path) = url.split_once('/').unwrap_or(("", ""));

    let db_conn = pool.get_ref().to_owned();

    let storage_path: String = db_conn
        .get()
        .unwrap()
        .query_row(
            "select storage_path from storages where storage_url = :storage_url;",
            named_params! {":storage_url":&storage_name},
            |row| row.get(0),
        )
        .unwrap();

    let path = format!("{}/{}", storage_path, _path);
    // log::info!("{:?}",_paths);
    if Path::new(&path).exists() {
        
        match &info.phoframe {
            Some(_text) =>{
                let text = _text.replace("+", " ").replace("|", "/");
                let _new_name = Path::new(_path).file_stem().and_then(|os_str| os_str.to_str()).unwrap();
                let new_path = format!("./tmp/{0}.jpg",_new_name);
                log::info!("{:?}  {:?}",text,new_path);
                let _handle = thread::spawn(move || {
                    add_frame(path,new_path.clone(),text,"LXGWWenKaiMono-Regular.ttf".to_string());
                });
                
                if Path::new(&new_path).exists() {
                    Ok(NamedFile::open(new_path).unwrap())
                }
                else{
                    Err(actix_web::error::ErrorNotFound("Image not found"))
                }  
            }
            None =>{
                Ok(NamedFile::open(path).unwrap())
            }
        }
    } else {
        Err(actix_web::error::ErrorNotFound("Image not found"))
    }
}

#[route("/raw2jpg", method = "POST")]
async fn raw2jpg(
    session: Session,
    pool: web::Data<Pool>,
    parames: web::Json<Parameters>,
) -> HttpResponse {
    match proces::raw2(parames, pool.get_ref().to_owned()) {
        Some(res) => HttpResponse::Ok().json(res),
        None => HttpResponse::NotFound().finish(),
    }
}

#[route("/save", method = "POST")]
async fn savejpg(
    session: Session,
    pool: web::Data<Pool>,
    index: web::Data<Index>,
    parames: web::Json<(String, i32)>,
) -> HttpResponse {
    // if let Ok(Some(userid)) = session.get::<String>("userid") {
    // println!("{:?}", parames);
    let _parames = parames.0;
    let _pool = pool.get_ref().to_owned();
    let db_conn = _pool.get().unwrap();

    let cached_path: String = db_conn
        .query_row(
            "\
        select \
        storage_cached.storage_path || paths_cached.path as cached_path
        FROM \
            images \
        LEFT JOIN \
            paths AS paths_cached ON images.cache_id = paths_cached.id \
        LEFT JOIN \
            storages AS storage_cached ON paths_cached.storage_id = storage_cached.id \
        where images.id = :id;",
            named_params! {":id":&_parames.1},
            |row| row.get(0),
        )
        .unwrap();

    // log::info!(
    //     "{} {}",
    //     format!("./tmp/{}", _parames.0),
    //     format!("{}/{}", cached_path, _parames.0)
    // );
    fs::rename(
        format!("./tmp/{}", _parames.0),
        format!("{}/{}", cached_path, _parames.0),
    );
    db_conn
        .execute(
            "UPDATE images SET cache_file_name = ?2 WHERE id = ?1",
            (&_parames.1, &_parames.0),
        )
        .unwrap();
    let _index = index.get_ref().to_owned();
    sync_sqlite_to_tantivy(&_pool, &_index);

    HttpResponse::Ok().body("")
}

#[route("/uplut", method = "POST")]
async fn update_lut(
    session: Session,
    pool: web::Data<Pool>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> Result<impl Responder, Error> {
    let db_conn = pool.get_ref().to_owned();
    if let Ok((luts_path, storage_id)) = db_conn.get().unwrap().query_row(
        "select storage_path,id from storages where storage_usage = 'luts';",
        [],
        |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())),
    ) {
        let luts_path: String = luts_path;
        let storage_id: i32 = storage_id;

        for f in form.files {
            match db_conn.get().unwrap().execute(
                "INSERT INTO luts (storage_id, lut_name, comment) VALUES (?1, ?2, ?2)",
                (&storage_id, &f.file_name.as_ref().unwrap()),
            ) {
                Ok(_) => {
                    let path = format!("{}/{}", luts_path, f.file_name.as_ref().unwrap());
                    log::info!("上传存储路径：{}",path);
                    // f.file.persist(path).unwrap();
                    let o_path = f.file.path().to_string_lossy().to_string();
                    match f.file.persist(&path) {
                        Ok(_) => {},
                        Err(e) => {
                                // 如果发生跨设备链接错误，使用复制操作
                            std::fs::copy(&o_path, &path).unwrap();
                            std::fs::remove_file(o_path).unwrap();
                            log::error!("error : {}",e.to_string());
                            // Ok(HttpResponse::Ok().body(format!("File uploaded to: {:?}", path)))
                            
                        }
                    }
                }
                Err(_e) => {
                    log::error!("{:?}", _e);
                    // Err(actix_web::error::ErrorUnauthorized("aaa"))
                }
            };
        }
        Ok(HttpResponse::Ok())
    } else {
        Err(actix_web::error::ErrorUnauthorized("aaa"))
    }
}

async fn authentication(
    session: Session,
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    // pre-processing
    // log::info!("vvddd");
    if let (Ok(Some(userid)), Ok(Some(passkey))) = (
        session.get::<String>("userid"),
        session.get::<String>("passkey"),
    ) {
        // log::info!("v: {} {}", userid, passkey);
        let key = HS256Key::from_bytes(&"key".try_into_bytes().unwrap());
        match key.verify_token::<NoCustomClaims>(&passkey, None) {
            Ok(_claims) => {
                let res = next.call(req).await?;
                Ok(res)
            }
            Err(_) => Err(actix_web::error::ErrorUnauthorized("no auth")),
        }
    } else {
        // Err(AuthenticationError::AuthRequest("Invalid input".to_string()).into())
        Err(actix_web::error::ErrorUnauthorized("no auth"))
    }
}

pub fn register(config: &mut web::ServiceConfig) {
    config
        .app_data(web::Data::new(create_schema()))
        .service(
            web::scope("/api")
                .wrap(middleware::from_fn(authentication))
                .service(check_auth)
                .service(graphql)
                .service(scans)
                .service(get_image)
                .service(raw2jpg)
                .service(savejpg)
                .service(update_lut),
        )
        .service(auth)
        .service(create_user)
        .service(graphql_playground);
}

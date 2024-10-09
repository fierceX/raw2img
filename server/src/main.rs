use actix_cors::Cors;
use actix_files::Files;
use actix_web::{cookie::Key, middleware::Logger, web::Data, App, HttpServer};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web_static_files::ResourceFiles;


mod db;
mod handlers;
mod schemas;
mod proces;

use self::{db::{create_tantivy_index,sync_sqlite_to_tantivy,get_db_pool}, handlers::register};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let pool = get_db_pool();
    let index = create_tantivy_index().unwrap();

    sync_sqlite_to_tantivy(&pool,&index);

    log::info!("starting HTTP server on port 8081");
    log::info!("GraphiQL playground: http://localhost:8081/graphiql");

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
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(false)
                    .build(),
            )
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}
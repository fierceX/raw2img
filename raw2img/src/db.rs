use std::path::Path;

use blake2::digest::consts::True;
use chrono::prelude::*;
use r2d2_sqlite::SqliteConnectionManager;
use raw::Myexif;
use tantivy::collector::Count;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::query::TermQuery;
use tantivy::schema::*;
use tantivy::DateTime;
use tantivy::query::FuzzyTermQuery;
use tantivy::tokenizer::NgramTokenizer;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};

use crate::schemas::image::row2img;
use crate::schemas::image::Image;

pub type Pool = r2d2::Pool<SqliteConnectionManager>;

pub fn get_db_pool(database:&str) -> Pool {
    let database = Path::new(database);
    let is_db = !database.exists();

    let manager = SqliteConnectionManager::file(database);
    
    let db = r2d2::Pool::new(manager).unwrap();
    if is_db{
        // manager.
        let ddd = db.get().unwrap();
        ddd.execute_batch(r#"
            -- 用户表
            CREATE TABLE `users` (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL,
                email TEXT NOT NULL,
                password TEXT NOT NULL,
                wb bool NOT NULL,
                half_size bool NOT NULL,
                quality BIGINT NOT NULL,
                lut_id BIGINT,
                UNIQUE(email)
            );

            -- 存储表
            CREATE TABLE storages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                storage_name TEXT NOT NULL,
                storage_path TEXT,
                storage_type TEXT NOT NULL,
                storage_url TEXT,
                ACCESS_KEY TEXT,
                SECRET_KEY TEXT,
                bucket_name TEXT,
                added_time DATETIME NOT NULL,
                storage_usage TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id),
                UNIQUE(storage_name)
            );


            -- 路径表
            CREATE TABLE paths (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                storage_id INTEGER NOT NULL,
                path TEXT NOT NULL,
                FOREIGN KEY (storage_id) REFERENCES storage(id),
                UNIQUE(storage_id,path)
            );

            -- 图片信息表

            CREATE TABLE images (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                path_id INTEGER NOT NULL,
                cache_id INTEGER,
                file_name TEXT NOT NULL,
                cache_file_name TEXT ,
                scan_time DATETIME NOT NULL,
                shooting_time DATETIME NOT NULL,
                file_size BIGINT NOT NULL,
                mime_type TEXT NOT NULL,
                exif TEXT,
                FOREIGN KEY (user_id) REFERENCES users(id),
                FOREIGN KEY (path_id) REFERENCES path(id),
                UNIQUE(path_id,file_name)
            );

            CREATE TABLE luts(
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                storage_id INTEGER NOT NULL,
                lut_name TEXT NOT NULL,
                comment TEXT NOT NULL,
                UNIQUE(storage_id,lut_name)
            );

            -- 图片完整路径视图表
            DROP VIEW IF EXISTS `images_view`;
            CREATE VIEW images_view AS
            SELECT 
                images.id,images.user_id,images.file_name,images.cache_file_name,images.scan_time,images.shooting_time,images.file_size,images.mime_type,images.exif,
                "/api/img/" || storage_original.storage_url || paths_original.path || "/" || images.file_name AS original_url,
                case storage_cached.storage_type
                when "local" then "/api/img/" || storage_cached.storage_url || paths_cached.path || "/" || images.cache_file_name
                else storage_cached.storage_url || paths_cached.path || "/" || images.cache_file_name
                end AS cached_url
            FROM 
                images
            LEFT JOIN 
                paths AS paths_original ON images.path_id = paths_original.id
            LEFT JOIN 
                paths AS paths_cached ON images.cache_id = paths_cached.id
            LEFT JOIN 
                storages AS storage_original ON paths_original.storage_id = storage_original.id
            LEFT JOIN 
                storages AS storage_cached ON paths_cached.storage_id = storage_cached.id;
        "#);
    }
    db
    
}


pub fn create_tantivy_index(index_path:&str) -> tantivy::Result<Index> {

    let mut schema_builder = Schema::builder();

    // let id_options = TextOptions::default()
    // .set_indexing_options(
    //     TextFieldIndexing::default()
    //         .set_tokenizer("raw")
    //         .set_index_option(IndexRecordOption::WithFreqsAndPositions)
    // )
    // .set_stored();

    let text_field_indexing = TextFieldIndexing::default()
        .set_tokenizer("ngram3")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let text_options = TextOptions::default()
        .set_indexing_options(text_field_indexing)
        .set_stored();

    schema_builder.add_text_field("file_name", text_options);
    schema_builder.add_text_field("cache_url", TEXT | STORED);
    schema_builder.add_text_field("original_url", TEXT | STORED);
    schema_builder.add_i64_field("image_id", INDEXED |STORED| FAST);
    schema_builder.add_i64_field("user_id", INDEXED |STORED| FAST);
    schema_builder.add_i64_field("focal_len", INDEXED |STORED| FAST);
    schema_builder.add_f64_field("iso", INDEXED |STORED| FAST);
    schema_builder.add_f64_field("aperture", INDEXED | STORED| FAST);
    schema_builder.add_f64_field("shutter", INDEXED | STORED | FAST);


    let opts = DateOptions::from(INDEXED)
        .set_stored()
        .set_fast()
        .set_precision(tantivy::schema::DateTimePrecision::Seconds);

    schema_builder.add_date_field("shooting_date", opts);

    let schema = schema_builder.build();


    let index_path = Path::new(index_path);

    let index = if index_path.exists() {
        Index::open_in_dir(index_path)?
    } else {
        std::fs::create_dir_all(index_path);
        let index = Index::create_in_dir(index_path, schema.clone())?;
        index
        .tokenizers()
        .register("ngram3", NgramTokenizer::new(1, 3, false).unwrap());
        let mut index_writer:IndexWriter = index.writer(50_000_000)?;
        index_writer.commit()?;
        index
    };
    index
        .tokenizers()
        .register("ngram3", NgramTokenizer::new(1, 3, false).unwrap());

    Ok(index)
}


pub fn sync_sqlite_to_tantivy(pool: &Pool, index: &Index) {
    let conn = pool.get().unwrap();
    let mut stmt = conn.prepare("select * from images_view;").unwrap();
    // let mut rows = stmt.query(params![])?;

    let images:Vec<Image> = stmt.query_map([],|row| {
        row2img(row)
    }).unwrap().into_iter().filter_map(Result::ok).collect();

    let schema = index.schema();
    let iso = schema.get_field("iso").unwrap();
    let focal_len = schema.get_field("focal_len").unwrap();
    let aperture = schema.get_field("aperture").unwrap();
    let shutter = schema.get_field("shutter").unwrap();
    let image_id = schema.get_field("image_id").unwrap();
    let user_id = schema.get_field("user_id").unwrap();
    let cache_url = schema.get_field("cache_url").unwrap();
    let original_url = schema.get_field("original_url").unwrap();
    let file_name = schema.get_field("file_name").unwrap();
    let shooting_date = schema.get_field("shooting_date").unwrap();


    let mut index_writer = index.writer(50_000_000).unwrap();

    for image in images.iter(){
        let reader = index.reader().unwrap();
        let searcher = reader.searcher();
        let query = TermQuery::new(
            Term::from_field_i64(image_id, image.id.into()),
            IndexRecordOption::Basic,
        );
        let count = searcher.search(&query, &Count).unwrap();

        if count > 0 {
            // 删除现有文档
            index_writer.delete_term(Term::from_field_i64(image_id, image.id.into()));
        }
        let mut _doc = TantivyDocument::default();

        if let Ok(exif) = serde_json::from_str(&image.exif){
            let _exif:Myexif = exif;
            _doc.add_text(file_name,image.file_name.clone());
        
            _doc.add_i64(image_id, image.id.into());
            _doc.add_i64(user_id, image.user_id.into());
            _doc.add_text(cache_url, image.cached_url.clone());
            _doc.add_text(original_url, image.original_url.clone());
    
            _doc.add_f64(aperture,_exif.aperture.into());
            _doc.add_f64(shutter,_exif.shutter.into());
            _doc.add_i64(focal_len,_exif.focal_len.into());
            _doc.add_f64(iso,_exif.iso.into());
           
            let naive_date = chrono::NaiveDateTime::parse_from_str(&image.shooting_time, "%Y-%m-%d %H:%M:%S").unwrap();
            let date_time = Local.from_local_datetime(&naive_date).unwrap();
    
            let dd = DateTime::from_timestamp_secs(date_time.timestamp());
            _doc.add_date(shooting_date,dd);
    
            index_writer.add_document(_doc);
        }
    }
    
    index_writer.commit().unwrap();
}
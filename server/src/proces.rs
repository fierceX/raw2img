use crate::db::{Pool,get_db_pool};
use crate::handlers::Parameters;
use actix_web::web;
use raw::raw_process;
use chrono::prelude::*;
use blake2;
use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use rusqlite::named_params;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};



pub fn scan_directory(dir: &Path, filter_list: &[&str],base_dir:&Path) -> HashMap<String, Vec<(String, String, u64)>> {
    let mut result = HashMap::new();

    if dir.is_dir() {
        for entry in dir.read_dir().expect("Failed to read directory") {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    // 递归扫描子目录
                    let sub_result = scan_directory(&path, filter_list,&base_dir);
                    for (key, value) in sub_result {
                        result.entry(key).or_insert_with(Vec::new).extend(value);
                    }
                } else if path.is_file() {
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    let relative_path = path.strip_prefix(base_dir).unwrap();
                    let parent_dir = format!("/{}",relative_path.parent().unwrap_or(Path::new("")).to_string_lossy().to_string());
                    let file_type = path.extension().unwrap_or_default().to_string_lossy().to_string();
                    let metadata = fs::metadata(&path).expect("Failed to get metadata");
                    let file_size = metadata.len();
                    result.entry(parent_dir)
                              .or_insert_with(Vec::new)
                              .push((file_name,file_type,file_size));
                }
            }
        }
    }

    result
}

pub fn scan_files(user_id:i32,pool:Arc<Mutex<Pool>>){
    let conn = pool.lock().unwrap().get().unwrap();

    let mut res = conn.prepare("select id,storage_name,storage_path from storages where user_id = :user_id and storage_type = :storage_type and storage_usage = 'source';").unwrap();
    let storages:Vec<(i32,String,String)> = res.query_map(named_params!{":user_id":&user_id,":storage_type":"local"},|row| {
        Ok((row.get(0).unwrap(),row.get(1).unwrap(),row.get(2).unwrap()
        ))
    }).unwrap().into_iter().filter_map(Result::ok).collect();
    for (_storage_id,_storage_name,_storage_path) in storages {
        let files = scan_directory(Path::new(&_storage_path), &[], Path::new(&_storage_path));
        for (path,file_names) in files {
            conn.execute(
                "INSERT OR IGNORE INTO paths (storage_id, path) VALUES (?1, ?2)",
                (&_storage_id, &path),
            ).unwrap();
            
            let _id:i32 = conn.query_row("select id from paths where path = :path and storage_id= :storage_id;", named_params!{":path":&path,":storage_id":&_storage_id}, |row| row.get(0)).unwrap();
            let now: DateTime<Utc> = Utc::now();
            println!("{} {}",_id,path);
            // 格式化时间
            let formatted_time = now.format("%Y-%m-%d %H:%M:%S").to_string();
            for (file_name,file_type,file_size) in file_names{
                conn.execute(
                    "INSERT OR IGNORE INTO images (user_id, path_id, file_name,scan_time,file_size,mime_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    (&user_id, &_id,&file_name,&formatted_time,&file_size,&file_type),
                ).unwrap();
                
                println!("{}",file_name);
            }
        }
    }
    raw2img(user_id,pool);
}

pub fn raw2(parames:web::Json<Parameters>,pool:Pool) -> Option<String>{
    let db_conn = pool.get().unwrap();
    println!("{:?}",parames);
    let original_path:String = db_conn.query_row("\
        select \
        storage_original.storage_path || paths_original.path as original_path
        FROM \
            images \
        LEFT JOIN \
            paths AS paths_original ON images.path_id = paths_original.id \
        LEFT JOIN \
            storages AS storage_original ON paths_original.storage_id = storage_original.id \
        where images.id = :id;"
        , named_params!{":id":&parames.id}, |row| row.get(0),).unwrap();

        // let lut_path:String = db_conn.get().unwrap().query_row("",
        let intput_file_path = format!("{}{}", original_path, parames.filename);

        if let Ok(_) = fs::metadata(intput_file_path.clone()) {
            let mut hasher = Blake2bVar::new(10).unwrap();

            let mut buf = [0u8; 10];
            hasher.update(
                format!(
                    "{}{}{}{}{}{}",
                    parames.id,
                    parames.filename,
                    parames.exp_shift,
                    parames.lut,
                    parames.threshold,
                    parames.wb
                )
                .as_bytes(),
            );
            hasher.finalize_variable(&mut buf).unwrap();

            // let _ = std::fs::create_dir_all(format!("./tmp/", dir_path));
            let out_file_name = format!("{}.jpg", base16ct::lower::encode_string(&buf));
            let out_file_path = format!(
                "./tmp/{}",
                out_file_name
            );

            if let Ok(_) = fs::metadata(out_file_path.clone()) {
                Some(format!("/tmp/{}",out_file_name))
            } else {
                let _ = raw_process(
                    intput_file_path,
                    out_file_path.clone(),
                    parames.lut.clone(),
                    parames.wb,
                    true,
                    parames.exp_shift as f32,
                    parames.threshold,
                    90,
                );
                Some(format!("/tmp/{}",out_file_name))
            }
        }
        else{
            None
        }
}

pub fn raw2img(user_id:i32,pool:Arc<Mutex<Pool>>){
    let conn = pool.lock().unwrap().get().unwrap();
    let mut res = conn.prepare("select images.id,file_name,mime_type,scan_time,storages.storage_path || paths.path || '/' || images.file_name as file_path from images left join paths on images.path_id = paths.id left join storages on paths.storage_id = storages.id where images.user_id = :user_id and storages.storage_type = :storage_type and images.cache_id is null;").unwrap();
    let images:Vec<(i32,String,String,String,String)> = res.query_map(named_params!{":user_id":&user_id,":storage_type":"local"},|row| {
        Ok((row.get(0).unwrap(),row.get(1).unwrap(),row.get(2).unwrap(),row.get(3).unwrap(),row.get(4).unwrap()
        ))
    }).unwrap().into_iter().filter_map(Result::ok).collect();
    println!("{:?}",images);
    let (lut_name,lut_path) = match conn.query_row("select lut_name,lut_path from users left join luts on users.lut_id = luts.id where id = :user_id;", named_params!{":user_id":&user_id}, |row| Ok((row.get(0).unwrap(),row.get(1).unwrap())),){
        Ok((_lut_name,_lut_path)) => (_lut_name,_lut_path),
        Err(_) => ("".to_string(),"".to_string())
    };

    let (storage_id,storage_path):(i32,String) = conn.query_row("select id,storage_path from storages where user_id = :user_id and storage_usage = 'cache';", named_params!{":user_id":&user_id}, |row| Ok((row.get(0).unwrap(),row.get(1).unwrap())),).unwrap();
    
    let now: DateTime<Utc> = Utc::now();
    // 格式化时间
    let formatted_time = now.format("%Y%m").to_string();

    let cache_path = format!("{}/{}",storage_path,formatted_time);
    let _cache_path = format!("/{}",formatted_time);

    if !std::path::Path::new(&cache_path).is_dir() {
        let _ = std::fs::create_dir_all(&cache_path);
    }

    conn.execute(
        "INSERT OR IGNORE INTO paths (storage_id, path) VALUES (?1, ?2)",
        (&storage_id, &_cache_path),
    ).unwrap();
    
    let cache_id:i32 = conn.query_row("select id from paths where path = :path and storage_id= :storage_id;", named_params!{":path":&_cache_path,":storage_id":&storage_id}, |row| row.get(0)).unwrap();
    println!("{:?}",cache_id);
    for (_id,_file_name,_type,_scan_time,_path) in images{
        println!("{}",_path);
        if let Ok(_) = fs::metadata(_path.clone()) {
            println!("aaaaa");
            let mut hasher = Blake2bVar::new(10).unwrap();

            let mut buf = [0u8; 10];
            hasher.update(
                format!(
                    "{}{}{}{}{}",
                    _id,
                    _file_name,
                    _type,
                    _scan_time,
                    lut_name
                )
                .as_bytes(),
            );
            hasher.finalize_variable(&mut buf).unwrap();
            
            let out_file_name = format!("{}.jpg",base16ct::lower::encode_string(&buf));
            let out_file_path = format!("{}/{}",cache_path,out_file_name);
            println!("{} {}",out_file_name,out_file_path);
            if let Ok(_exif) = raw_process(
                _path,
                out_file_path,
                lut_path.clone(),
                true,
                true,
                -3.0,
                -3,
                90,
            ){
                let _exif_json = serde_json::to_string(&_exif).unwrap();
                conn.execute(
                    "UPDATE images SET cache_id = ?2,cache_file_name = ?3, exif = ?4 WHERE id = ?1",
                    (&_id, &cache_id,&out_file_name,&_exif_json),
                ).unwrap();
            }
            
        }
    }
}
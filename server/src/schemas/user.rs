use chrono::Local;
use juniper::{graphql_object, GraphQLInputObject};
use raw::Myexif;
use tantivy::{collector::TopDocs, query::QueryParser, schema::Value, TantivyDocument};
use crate::schemas::{root::Context,image::Image,image::row2img,storage::Storage,storage::row2storage};
use rusqlite::Error;

#[derive(Default, Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub lut_id: i32,
    pub wb: bool,
    pub half_size: bool,
    pub quality: i32,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "User Input")]
pub struct UserInput {
    pub name: String,
    pub email: String,
    pub password: String,
    pub lut_id: i32,
    pub wb: bool,
    pub half_size: bool,
    pub quality: i32,
}

#[graphql_object(Context = Context)]
impl User {
    fn id(&self) -> &i32 {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn email(&self) -> &str {
        &self.email
    }
    fn lut_id(&self) -> &i32{
        &self.lut_id
    }
    fn wb(&self) -> &bool{
        &self.wb
    }
    fn half_size(&self) -> &bool{
        &self.half_size
    }
    fn quality(&self) -> &i32{
        &self.quality
    }

    fn storages(&self, context: &Context) -> Vec<Storage> {
        let conn = context.db_pool.get().unwrap();

        let mut res = conn.prepare("SELECT * FROM storages WHERE user_id = :user_id").unwrap();
        res.query_map(&[(":user_id", &self.id)],|row| {
            row2storage(row)
        }).unwrap().into_iter().filter_map(Result::ok).collect()
    }
    fn images(&self, context: &Context) -> Vec<Image> {
        let conn = context.db_pool.get().unwrap();

        let mut res = conn.prepare("select * from images_view where user_id = :user_id and images.cache_id is null;").unwrap();
        res.query_map(&[(":user_id",&self.id)],|row| {
            row2img(row)
        }).unwrap().into_iter().filter_map(Result::ok).collect()
    }

    fn search(&self,context: &Context,query: String) -> Vec<Image> {
        let searcher =context.index.reader().unwrap().searcher();

        let schema = context.index.schema();
        let file_name = schema.get_field("file_name").unwrap();
        let focal_len = schema.get_field("focal_len").unwrap();
        let iso = schema.get_field("iso").unwrap();
        let aperture = schema.get_field("aperture").unwrap();
        let shutter = schema.get_field("shutter").unwrap();
        let image_id = schema.get_field("image_id").unwrap();
        let user_id = schema.get_field("user_id").unwrap();
        let cache_url = schema.get_field("cache_url").unwrap();
        let shooting_date = schema.get_field("shooting_date").unwrap();

        let mut query_parser = QueryParser::for_index(&context.index, vec![file_name]);
        // query_parser.set_field_fuzzy(file_name,false,2,true);
        
        let query_str = format!("user_id:{0} AND ({1})",self.id,query);
        println!("{}",query_str);
        let queryq = query_parser.parse_query(&query_str).unwrap();



        let top_docs = searcher.search(&queryq, &TopDocs::with_limit(1000)).unwrap();

        println!("{:?}",top_docs);

        let images = top_docs.iter().map(|(_s,_d)|{
            let retrieved_doc: TantivyDocument = searcher.doc(*_d).unwrap();
            let _time_timestamp = retrieved_doc.get_first(shooting_date).unwrap().as_datetime().unwrap().into_timestamp_millis();
            let _time:chrono::DateTime<Local> = chrono::DateTime::from(chrono::DateTime::from_timestamp_millis(_time_timestamp).unwrap());
            let _time_str = _time.format("%Y-%m-%d %H:%M:%S").to_string();

            let _exif:Myexif = Myexif{
                iso:retrieved_doc.get_first(iso).unwrap().as_f64().unwrap() as f32,
                aperture:retrieved_doc.get_first(aperture).unwrap().as_f64().unwrap() as f32,
                shutter:retrieved_doc.get_first(shutter).unwrap().as_f64().unwrap() as f32,
                focal_len:retrieved_doc.get_first(focal_len).unwrap().as_i64().unwrap() as u16,
                shooting_date:_time_str.clone(),
            };

            let _exif_str = serde_json::to_string(&_exif).unwrap();

            Image{
                id:retrieved_doc.get_first(image_id).unwrap().as_i64().unwrap() as i32,
                user_id:retrieved_doc.get_first(user_id).unwrap().as_i64().unwrap() as i32,
                file_name:retrieved_doc.get_first(file_name).unwrap().as_str().unwrap().to_string(),
                cache_file_name:"".to_string(),
                scan_time:"".to_string(),
                shooting_time:_time_str,
                file_size:-1,
                mime_type:"".to_string(),
                exif:_exif_str,
                original_url:"".to_string(),
                cached_url:retrieved_doc.get_first(cache_url).unwrap().as_str().unwrap().to_string(),
            }

        }).collect();
        images
    }
}

pub fn row2user(row:&rusqlite::Row<'_>) -> Result<User, Error>{
    Ok(User{
        id: row.get(0).unwrap(),
        name: row.get(1).unwrap(),
        email: row.get(2).unwrap(),
        wb: row.get(4).unwrap(),
        half_size: row.get(5).unwrap(),
        quality: row.get(6).unwrap(),
        lut_id: row.get(7).unwrap_or(-1),
    })
}
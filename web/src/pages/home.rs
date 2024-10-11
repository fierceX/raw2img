use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sycamore::{futures::spawn_local_scoped, web::html::img};
use sycamore::prelude::*;
use web_sys::{HtmlInputElement, HtmlOptionElement};
use graphql_client::{reqwest::post_graphql, GraphQLQuery};


#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
struct Parameters {
    id:i32,
    filename: String,
    lut: String,
    wb: bool,
    exp_shift: f64,
    threshold: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Myexif {
    iso: f32,
    aperture: f32,
    shutter: f32,
    focal_len: i32,
    pub shooting_date:String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Image {
    id:i32,
    exif:Myexif,
    filename: String,
    url: String,
}


#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas.json",
    query_path = "images.graphql",
    response_derives = "Debug",
)]
pub struct ImagesQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas.json",
    query_path = "search.graphql",
    response_derives = "Debug",
)]
pub struct ImagesSearch;


#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas.json",
    query_path = "storage.graphql",
    response_derives = "Debug",
)]
pub struct StorageQuery;



#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas.json",
    query_path = "luts.graphql",
    response_derives = "Debug",
)]
pub struct LutsQuery;


async fn getluts(url:&str) -> Vec<(String, String)> {
    // let base_url = web_sys::window().unwrap().location().origin().unwrap();
    // let url = format!("{}/api/graphql", base_url);
    // let url = format!("http://127.0.0.1:8081/api/graphql");
    let client = reqwest::Client::new();
    let variables = luts_query::Variables{};
    let response_body = 
        post_graphql::<LutsQuery, _>(&client, url, variables).await.unwrap();
    // log::info!("{:?}",response_body);
    let response_data: luts_query::ResponseData = response_body.data.expect("missing response data");
    response_data.luts.iter().map(|x| (format!("{}/{}",x.path.clone(),x.lut_name.clone()),x.lut_name.clone())).collect()
}

async fn getrawfiles(user_id:i32, url:&str) -> (Vec<Image>,Vec<(String, Vec<(usize, Image)>)>) {
    // let base_url = web_sys::window().unwrap().location().origin().unwrap();
    // let url = format!("{}/api/graphql", base_url);
    // let url = format!("http://127.0.0.1:8081/api/graphql");
    
    let client = reqwest::Client::new();
    let variables = images_query::Variables {
        id: user_id.to_string(),
    };
    let response_body = 
        post_graphql::<ImagesQuery, _>(&client, url, variables).await.unwrap();
    // log::info!("{:?}",response_body);
    let response_data: images_query::ResponseData = response_body.data.expect("missing response data");
    let mut image_list: Vec<Image> = response_data.user.images.iter().map(|x| {
        
        if let Ok(_exif) = serde_json::from_str(&x.exif) {
        // log::info!("{:?}",x.exif);
        
            Image{
            id:x.id as i32,
            exif:_exif,
            filename: x.file_name.clone(),
            url: x.cached_url.clone(),}
        }
        else{
            Image{
                id:x.id as i32,
                exif:Myexif{iso:0.0,aperture:0.0,shutter:0.0,focal_len:0,shooting_date:"1900-01-01".to_string()},
                filename: x.file_name.clone(),
                url: x.cached_url.clone(),}
        }

    }).collect();
    // image_list.sort_by_key(|p|p.1.exif.shooting_date);
    image_list.sort_by(|a,b|b.exif.shooting_date.cmp(&a.exif.shooting_date));
    (image_list.clone(),image_list.iter().enumerate().map(|(i,x)|{
        (i,x.clone())
    }).into_iter()
    .fold(HashMap::new(), |mut map, image| {
        let date = image.1.exif.shooting_date.split_once(" ").unwrap().0.to_string();
        map.entry(date).or_insert_with(Vec::new).push(image);
        map
    }).into_iter()
    .collect())

}


async fn search(user_id:i32, url:&str, query:&str) -> (Vec<Image>,Vec<(String, Vec<(usize, Image)>)>) {
    // let base_url = web_sys::window().unwrap().location().origin().unwrap();
    // let url = format!("{}/api/graphql", base_url);
    // let url = format!("http://127.0.0.1:8081/api/graphql");
    
    let client = reqwest::Client::new();
    let query_str = if query != ""{
        query
    }
    else{
        "*"
    };
    let variables = images_search::Variables {
        id: user_id.to_string(),
        query:query_str.to_string(),
    };
    let response_body = 
        post_graphql::<ImagesSearch, _>(&client, url, variables).await.unwrap();
    // log::info!("{:?}",response_body);
    let response_data: images_search::ResponseData = response_body.data.expect("missing response data");
    let mut image_list: Vec<Image> = response_data.user.search.iter().map(|x| {
        
        if let Ok(_exif) = serde_json::from_str(&x.exif) {
        // log::info!("{:?}",x.exif);
        
            Image{
            id:x.id as i32,
            exif:_exif,
            filename: x.file_name.clone(),
            url: x.cached_url.clone(),}
        }
        else{
            Image{
                id:x.id as i32,
                exif:Myexif{iso:0.0,aperture:0.0,shutter:0.0,focal_len:0,shooting_date:"1900-01-01".to_string()},
                filename: x.file_name.clone(),
                url: x.cached_url.clone(),}
        }

    }).collect();
    // image_list.sort_by_key(|p|p.1.exif.shooting_date);
    image_list.sort_by(|a,b|b.exif.shooting_date.cmp(&a.exif.shooting_date));
    // image_list.iter().enumerate().map(|(i,x)|{
    //     (i,x.clone())
    // }).collect()

    (image_list.clone(),image_list.iter().enumerate().map(|(i,x)|{
        (i,x.clone())
    }).into_iter()
    .fold(HashMap::new(), |mut map, image| {
        let date = image.1.exif.shooting_date.split_once(" ").unwrap().0.to_string();
        map.entry(date).or_insert_with(Vec::new).push(image);
        map
    }).into_iter()
    .collect())
    // image_list

    // println!("{:?}",response_data.user.images[1].cache_file_name);

    // log::info!("{:?}",response_data.user.images);
}


async fn get_jpg(params: Parameters,base_url:&str) -> String {
    // let base_url = web_sys::window().unwrap().location().origin().unwrap();
    // let url = format!("http://127.0.0.1:8081/api/raw2jpg");
    // log::info!("{:?}", params);
    let url = format!("{}/api/raw2jpg", base_url);
    reqwest::Client::new()
        .post(&url)
        .json(&params)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
    // format!("{}/api/{}", base_url, body)
}

async fn save_jpg(url_file: String, image_id: i32, base_url:&str) {
    // let base_url = web_sys::window().unwrap().location().origin().unwrap();
    // let url = format!("http://127.0.0.1:8081/api/save");
    // log::info!("{:?}", params);
    let url = format!("{}/api/save", base_url);
    reqwest::Client::new()
        .post(&url)
        .json(&(url_file, image_id))
        .send()
        .await
        .unwrap();
    // format!("{}/api/{}", base_url, body)
}

#[component]
pub async fn Body<G: Html>(cx: Scope<'_>) -> View<G> {
    let base_url = web_sys::window().unwrap().location().origin().unwrap();
    // let base_url = "http://127.0.0.1:8081".to_string();
    let graphql_url = format!("{}/api/graphql",base_url);

    
    let img_url = create_signal(cx, String::new());
    let file_name = create_signal(cx, String::new());
    let exp_string = create_signal(cx, String::new());
    let exp_shift = create_signal(cx, String::new());
    let exp_shift_flag = create_signal(cx, true);
    let exp_shift_ref = create_node_ref(cx);
    exp_shift.set("0".to_string());

    let threshold = create_signal(cx, String::new());
    let threshold_flag = create_signal(cx, true);
    let threshold_ref = create_node_ref(cx);
    threshold.set("0".to_string());

    let lut_ref = create_node_ref(cx);

    let images = create_signal(cx, Vec::new());

    let images_list = create_signal(cx, Vec::new());

    let user_id = use_context::<RcSignal<i32>>(cx);

    let (_images_list,_images) = getrawfiles(*user_id.get(),&graphql_url).await;
    let mut _images_:Vec<(String,&Signal<Vec<(usize,Image)>>)> = Vec::new();
    for i in _images.iter(){
        let _x = create_signal(cx, Vec::new());
        _x.set(i.1.clone());
        _images_.push((i.0.clone(),_x));
    }

    images.set(_images_);
    images_list.set(_images_list);

    

    let auto_wb_ref = create_node_ref(cx);
    let wb_label = create_signal(cx, String::new());

    let loading = create_signal(cx, false);

    wb_label.set("自动白平衡".to_string());

    // 当前显示的图片索引
    let current_index = create_signal(cx, 0);

    // 图片是否放大的状态
    let is_zoomed = create_signal(cx, false);

    let is_edit = create_signal(cx, false);

    let luts = create_signal(cx, getluts(&graphql_url).await);

    let base_url_c = create_signal(cx,base_url);
    let graphql_url_c = create_signal(cx,graphql_url);


    let query_value = create_signal(cx, String::new());

    let start_date = create_signal(cx, "".to_string());
    let end_date = create_signal(cx, "".to_string());

    let selected_items = create_signal(cx, vec![]);

    let buquan_disply = create_signal(cx, "none;");

    let input_value = create_signal(cx, "".to_string());



    // 处理图片点击事件
    let handle_image_click = move |index: usize| {
        current_index.set(index);
        is_zoomed.set(!*is_zoomed.get());
    };

    // 处理左切换按钮点击事件
    let handle_prev_click = move |_| {
        // log::info!("{:?}", *current_index.get());
        current_index.set((*current_index.get() + images.get().len() - 1) % images.get().len());
    };

    // 处理右切换按钮点击事件
    let handle_next_click = move |_| {
        // log::info!("{:?}", *current_index.get());
        current_index.set((*current_index.get() + 1) % images.get().len());
    };

    let bat = move |_| {
        spawn_local_scoped(cx, async move {
            loading.set(true);
            let lut = lut_ref
                .get::<DomNode>()
                .unchecked_into::<HtmlOptionElement>()
                .value();
            // let filename = raw_ref
            //     .get::<DomNode>()
            //     .unchecked_into::<HtmlOptionElement>()
            //     .value();
            let filename = images_list.get()[*current_index.get()].filename.clone();
            let image_id = images_list.get()[*current_index.get()].id.clone();
            let wb = auto_wb_ref
                .get::<DomNode>()
                .unchecked_into::<HtmlInputElement>()
                .checked();
            let threshold_ = if *threshold_flag.get() {
                -1
            } else {
                threshold.get().to_string().parse::<i32>().unwrap()
            };
            let exp_shift_ = if *exp_shift_flag.get() {
                -3.0
            } else {
                exp_shift.get().to_string().parse::<f64>().unwrap()
            };

            let exp_string_ = format!(
                "lut: {} wb: {} exp_shift: {} threshold: {}",
                lut,
                if wb { "auto" } else { "camera" },
                if *exp_shift_flag.get() {
                    "auto".to_string()
                } else {
                    exp_shift.get().to_string()
                },
                if *threshold_flag.get() {
                    "auto".to_string()
                } else {
                    threshold.get().to_string()
                }
            );
            let filename_ = filename.clone();
            img_url.set(
                get_jpg(Parameters {
                    id:image_id,
                    filename,
                    lut,
                    wb,
                    exp_shift: exp_shift_,
                    threshold: threshold_,
                },base_url_c.get().as_str())
                .await,
            );
            file_name.set(filename_);
            exp_string.set(exp_string_);
            loading.set(false);
        })
    };

    let save =  move |_| {
        spawn_local_scoped(cx, async move {
            let url_file = img_url
                .get()
                .to_string()
                .split("/")
                .last()
                .unwrap()
                .to_string();
            let image_id = images_list.get()[*current_index.get()].id.clone();
            save_jpg(url_file, image_id,base_url_c.get().as_str()).await;
            let (_images_list,_images) = getrawfiles(*user_id.get(),graphql_url_c.get().as_str()).await;
            let mut _images_:Vec<(String,&Signal<Vec<(usize,Image)>>)> = Vec::new();
            for i in _images.iter(){
                let _x = create_signal(cx, Vec::new());
                _x.set(i.1.clone());
                _images_.push((i.0.clone(),_x));
            }

            images.set(_images_);
            images_list.set(_images_list);
            is_edit.set(false);
        })
    };

    let image_search =  move |_| {
        spawn_local_scoped(cx, async move {
            let url_file = img_url
                .get()
                .to_string()
                .split("/")
                .last()
                .unwrap()
                .to_string();
            // let file_name = images.get()[*current_index.get()].1.filename.clone();
            // let image_id = images.get()[*current_index.get()].1.id.clone();
            // save_jpg(url_file, image_id,base_url_c.get().as_str()).await;

            let (_images_list,_images) = search(*user_id.get(),graphql_url_c.get().as_str(),query_value.get().as_str()).await;
            let mut _images_:Vec<(String,&Signal<Vec<(usize,Image)>>)> = Vec::new();
            for i in _images.iter(){
                let _x = create_signal(cx, Vec::new());
                _x.set(i.1.clone());
                _images_.push((i.0.clone(),_x));
            }
            images.set(_images_);
            images_list.set(_images_list);
            is_edit.set(false);
        })
    };

    let candidate = move |_| {
        buquan_disply.set("");
        let input = input_value.get();
        let mut ddd: Vec<(String, String)> = Vec::new();
        if input != "".to_string().into() {
            if input.parse::<i32>().is_ok() {
                let _x = format!("focal_len:{}", input);
                ddd.push((_x.clone(), _x));
            }
            if input.parse::<f32>().is_ok() {
                let _x = format!("iso:{}", input);
                ddd.push((_x.clone(), _x));
                let _x = format!("aperture:{}", input);
                ddd.push((_x.clone(), _x));
                let _x = format!("shutter:{}", input);
                ddd.push((_x.clone(), _x));
            }
            let _x = format!("file_name:{}", input);
            ddd.push((_x.clone(), _x));
        }
        selected_items.set(ddd);
    };

    let add_item = move |item: &String| {
        let _query_value = query_value.get();
        if _query_value == "".to_string().into() {
            query_value.set(format!("{0}", item));
        } else {
            query_value.set(format!("{0} AND {1}", _query_value, item));
        }

        input_value.set("".to_string());
        selected_items.set(vec![]);
    };

    
    view! {cx,

        div {
            details(){
                summary(){"搜索"}
                article(){
                    header(){
                        div(class="grid"){
                            input(type="date",bind:value=start_date){}
                            input(type="date",bind:value=end_date,on:change=move |_|{
                                if end_date.get() > start_date.get(){
                                    add_item(&format!("shooting_date:[{0}T00:00:00+08:00 TO {1}T00:00:00+08:00]",start_date.get(),end_date.get()));
                                }
                                else{
                                    end_date.set("".to_string())
                                }
                            }){}

                        div {
                            input(
                                placeholder="Type to search...",
                                bind:value=input_value,
                                on:input=candidate,
                                on:blur = |_|{
                                    buquan_disply.set("none;");
                                }
                            )
                            div(){
                                ul {
                                    Keyed (
                                        iterable=selected_items,
                                        view= move |cx, (item_0,item_1)| view! { cx,
                                            li(on:click=move |_| add_item(&item_0)) { (item_1) }
                                        },
                                        key=|item| item.clone(),
                                    )
                                    }
                                }  
                            }
                        }
                    }

            
                    input(value=query_value.get(),readonly=true)
                    footer(style="display: flex;justify-content: center;align-items: center;"){
                        button(on:click=move |_| query_value.set("".to_string())){"Clear"}
                        button(on:click=image_search){"Search"}
                    }
                }
            }
        }


        div(class="row",hidden=*is_zoomed.get() || *is_edit.get()){
            Indexed(
                iterable=images,
                view=move |cx, (date,image)|
                view! {cx,
                    div(class="row"){
                        h6(){(date)}
                    div(class="custom-grid"){
                        
                    Indexed(
                        iterable=image,
                        view=move |cx, (index,aimage)|
                        view! {cx,
                                article(){

                                    header(style="display: flex; justify-content: space-between;"){
                            
                                                i(class="bx bx-wrench",style="margin-right: 20px;",on:click=move |_| {
                                                    current_index.set(index);
                                                    let _image = images_list.get()[index].clone();
                                                    is_edit.set(true);
                                                    file_name.set(_image.filename);
                                                    img_url.set(_image.url);
                                                })
                                                i(class="bx bx-info-circle")
                                                
                                    }
                                    img(style="display: block;margin-left: auto;margin-right: auto;",loading="lazy",src=aimage.url,on:click=move |_| handle_image_click(index))
                                    footer(){
                                        small(){
                                            i(class="bx bx-aperture",style="margin-right: 20px;"){(aimage.exif.aperture)}
                                            i(class="bx bx-time-five",style="margin-right: 20px;"){"1/"((1.0/aimage.exif.shutter).round())}
                                            i(class="bx bx-album",style="margin-right: 20px;"){(aimage.exif.focal_len)}
                                            i(class="bx bx-adjust"){(aimage.exif.iso)}
                                            }
                                    }
                                }
                        }
                    )
                }
            }
                }
            )
        }

        (if *is_zoomed.get() {
                view! { cx,
                    div(class="image-container") {
                        img(src=images_list.get()[*current_index.get()].url,class="zoomed", on:click=move |_| is_zoomed.set(false))
                        div(class="button-container-prev"){
                            button(class="prev-button", on:click=handle_prev_click) { "" }
                        }
                        div(class="button-container-next"){
                            button(class="next-button", on:click=handle_next_click) { "" }
                        }
                    }
                }
            }
            else{
                view!{cx,

                }
            })
        dialog(open=*is_edit.get()){
            article(style="width: 100%; max-width: 80%"){
                header(){
                    button(rel="prev",on:click= move |_| is_edit.set(false)){}
                    p(){"自定义转换"}
                }

            div(class="grid",style="grid-template-columns:2fr 1fr;") {
            div(style="display: flex;justify-content: center;align-items: center;"){
                article(){
                    header(){(file_name.get())}
                    img(src = img_url.get())
                    footer(){small(){i(){(exp_string.get())}}}
                }
                }
            div(){
                fieldset(class="grid"){
                    article(){
                        header(){
                            select(ref=lut_ref,aria-label="选择Lut"){
                                option(selected=true,disabled=true,value="No Lut"){"选择Lut文件"}
                                Indexed(
                                    iterable=luts,
                                    view=|cx, x|
                                    view! {cx,
                                        option(value = x.0){(x.1)}
                                        },
                                    )
                                }
                        }
                    input(ref=auto_wb_ref,type="checkbox",role="switch",checked=true,on:click = move |_| {
                            let wb = auto_wb_ref.get::<DomNode>().unchecked_into::<HtmlInputElement>().checked();
                            if wb{
                                wb_label.set("自动白平衡".to_string());
                            }
                            else{
                                wb_label.set("相机白平衡".to_string());
                            }
                        })
                        label(){(wb_label.get())}
                    }
                    }


                fieldset(){
                    article(){
                        header(){"曝光补偿"}
                    fieldset(class="grid"){
                        input(bind:value=exp_shift,type="range",min="-2",max="3",step="0.1",disabled=*exp_shift_flag.get())
                        label(){(exp_shift.get())}
                        }
                    input(ref=exp_shift_ref,type="checkbox",role="switch",checked=true,on:click = move |_|{
                        let vvvw = exp_shift_ref.get::<DomNode>();
                        let vvaaw = vvvw.unchecked_into::<HtmlInputElement>().checked();
                        exp_shift_flag.set(vvaaw);
                    })"使用自动曝光补偿"

                    }
                }

                fieldset(){
                        article(){
                        header(){legend(){"降噪"}}
                    fieldset(class="grid"){
                    input(bind:value=threshold,type="range",min="0",max="1000",step="10",disabled=*threshold_flag.get())
                    label(){(threshold.get())}
                    }

                    input(ref=threshold_ref,type="checkbox",role="switch",checked=true,on:click = move |_|{
                        let vvvw = threshold_ref.get::<DomNode>();
                        let vvaaw = vvvw.unchecked_into::<HtmlInputElement>().checked();
                        threshold_flag.set(vvaaw);
                    })"使用自动降噪"

                }

                }
            }
        }
        footer(style="display: flex;justify-content: center;align-items: center;"){
            button(aria-busy=*loading.get(),on:click= bat) { "处理" }
            button(on:click= save) { "保存" }
            }
        }
    }
    }
}
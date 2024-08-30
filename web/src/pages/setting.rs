
use serde::{Deserialize, Serialize};
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use web_sys::{HtmlInputElement, HtmlOptionElement};
use graphql_client::{reqwest::post_graphql, GraphQLQuery};



#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas.json",
    query_path = "update_user.graphql",
    response_derives = "Debug",
)]
pub struct UpdateUser;


#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas.json",
    query_path = "queryuser.graphql",
    response_derives = "Debug",
)]
pub struct UserQuery;


#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas.json",
    query_path = "create_storage.graphql",
    response_derives = "Debug",
)]
pub struct CreateStorage;



#[derive(Default, Debug, Clone, PartialEq)]
pub struct Storage {
    pub id: i32,
    pub storage_name: String,
    pub storage_path: String,
    pub storage_type: String,
    pub storage_url: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket_name: String,
    pub added_time: String,
    pub storage_usage: String,
}


use update_user::UserInput;
use create_storage::StorageInput;

use crate::{pages::home::{luts_query, LutsQuery}, User};

async fn getuser(user_id:i32,url: &str) -> (UserInput,Vec<(usize,Storage)>) {
    // let base_url = web_sys::window().unwrap().location().origin().unwrap();
    // let url = format!("{}/api/graphql", base_url);
    // let url = format!("http://127.0.0.1:8081/api/graphql");
    
    let client = reqwest::Client::new();


    let variables = user_query::Variables {
        id: user_id.to_string()
    };

    
    let response_body = 
        post_graphql::<UserQuery, _>(&client, url, variables).await.unwrap();
    // log::info!("{:?}",response_body);
    let response_data: user_query::ResponseData = response_body.data.expect("missing response data");
    let stoarges = response_data.user.storages.iter().enumerate().map(|(i,x)| 
        (i,Storage{
            id:x.id.clone() as i32,
            storage_name:x.storage_name.clone(),
            storage_path:x.storage_path.clone(),
            storage_type:x.storage_type.clone(),
            storage_url:x.storage_url.clone(),
            access_key:x.access_key.clone(),
            secret_key:x.secret_key.clone(),
            bucket_name:x.bucket_name.clone(),
            added_time:x.added_time.clone(),
            storage_usage:x.storage_usage.clone(),
    })).collect();
    let user = UserInput{
        name:response_data.user.name.clone(),
        email:response_data.user.email.clone(),
        wb:response_data.user.wb.clone(),
        half_size:response_data.user.half_size.clone(),
        quality:response_data.user.quality.clone(),
        lut_id:response_data.user.lut_id.clone(),
        password: "".to_string(),
    };
    (user,stoarges)
}

async fn updateuser(user_id:i32,user:UserInput, url:&str) {
    // let base_url = web_sys::window().unwrap().location().origin().unwrap();
    // let base_url = "http://127.0.0.1:8081";
    let client = reqwest::Client::new();
    // let url = format!("{}/api/graphql", base_url);

    let variables = update_user::Variables {
        id: user_id.to_string(),
        user,
    };

    let response_body = 
        post_graphql::<UpdateUser, _>(&client, url, variables).await.unwrap();
    // log::info!("{:?}",response_body);
}


async fn createstorage(storage:StorageInput,url:&str) {
    // let base_url = web_sys::window().unwrap().location().origin().unwrap();
    // let base_url = "http://127.0.0.1:8081";
    let client = reqwest::Client::new();
    // let url = format!("{}/api/graphql", base_url);

    let variables = create_storage::Variables {
        storage,
    };

    let response_body = 
        post_graphql::<CreateStorage, _>(&client, url, variables).await.unwrap();
    // log::info!("{:?}",response_body);
}


async fn getluts(url: &str) -> Vec<(usize, String)> {
    // let base_url = web_sys::window().unwrap().location().origin().unwrap();
    // let url = format!("{}/api/graphql", base_url);
    // let url = format!("http://127.0.0.1:8081/api/graphql");
    let client = reqwest::Client::new();
    let variables = luts_query::Variables{};
    let response_body = 
        post_graphql::<LutsQuery, _>(&client, url, variables).await.unwrap();
    // log::info!("{:?}",response_body);
    let response_data: luts_query::ResponseData = response_body.data.expect("missing response data");
    response_data.luts.iter().map(|x| (x.id as usize,x.lut_name.clone())).collect()
}


async fn scan_files(user_id:i32,base_url:&str) {
    // let base_url = web_sys::window().unwrap().location().origin().unwrap();
    // let url = format!("http://127.0.0.1:8081/api/scan");
    // log::info!("{:?}", params);
    let url = format!("{}/api/scan", base_url);
    reqwest::Client::new()
        .post(&url)
        .json(&user_id)
        .send()
        .await
        .unwrap();
    // format!("{}/api/{}", base_url, body)
}

#[component]
pub async fn Body<G: Html>(cx: Scope<'_>) -> View<G> {
    let user_id = use_context::<RcSignal<i32>>(cx);

    let base_url = web_sys::window().unwrap().location().origin().unwrap();
    // let base_url = "http://127.0.0.1:8081".to_string();
    let graphql_url = format!("{}/api/graphql",base_url);

    // let user_id = 1;
    let (_user,_storages) = getuser(*user_id.get(),&graphql_url).await;
    // let img_url = create_signal(cx, String::new());
    let user = create_signal(cx, _user);
    let storages = create_signal(cx,_storages);

    let quality = create_signal(cx,user.get().quality.to_string());

    let auto_wb_ref = create_node_ref(cx);
    let wb_label = create_signal(cx, String::new());

    let auto_hf_ref = create_node_ref(cx);
    let hf_label = create_signal(cx, String::new());
    // hf_label.set("")

    let edit_storage = create_signal(cx, false);
    
    let lut_ref = create_node_ref(cx);

    

    let luts = create_signal(cx, getluts(&graphql_url).await);

    let upfile_ref = create_node_ref(cx);

    let loading = create_signal(cx, false);

    let base_url_c = create_signal(cx,base_url);
    let graphql_url_c = create_signal(cx,graphql_url);

    // let loading = create_signal(cx, false);
    if user.get().wb{
        wb_label.set("自动白平衡".to_string());
    }
    else{
        wb_label.set("相机白平衡".to_string());
    }

    if user.get().half_size{
        hf_label.set("半尺寸".to_string());
    }
    else{
        hf_label.set("原尺寸".to_string());
    }

    let storage_name = create_signal(cx, String::new());
    let storage_path = create_signal(cx, String::new());
    // let storage_use = create_signal(cx, String::new());
    let storage_use_ref = create_node_ref(cx);


    let bat = move |_| {
        spawn_local_scoped(cx, async move {
            // loading.set(true);
            let lut_id = lut_ref
                .get::<DomNode>()
                .unchecked_into::<HtmlOptionElement>()
                .value().parse().unwrap_or(-1);

            let wb = auto_wb_ref
                .get::<DomNode>()
                .unchecked_into::<HtmlInputElement>()
                .checked();
            let hf = auto_hf_ref
                .get::<DomNode>()
                .unchecked_into::<HtmlInputElement>()
                .checked();
            // user.set(value)
            let q = quality.get().clone();
            let _user = UserInput{
                name:user.get().name.clone(),
                email:user.get().email.clone(),
                password:"".to_string(),
                lut_id:lut_id as i64,
                wb:wb,
                half_size:hf,
                quality:q.parse::<i64>().unwrap(),
            };
            updateuser(*user_id.get(), _user,graphql_url_c.get().as_str()).await;
        })
    };

    let add_storage = move |_|{
        spawn_local_scoped(cx, async move {
            let storage_use = storage_use_ref
                .get::<DomNode>()
                .unchecked_into::<HtmlOptionElement>()
                .value();
            createstorage(StorageInput{
                user_id:*user_id.get() as i64,
                storage_name:storage_name.get().to_string(),
                storage_path:storage_path.get().to_string(),
                storage_type:"local".to_string(),
                storage_url:storage_name.get().to_string(),
                access_key:"".to_string(),
                secret_key:"".to_string(),
                bucket_name:"".to_string(),
                storage_usage:storage_use,
            },graphql_url_c.get().as_str()).await;
            let (_,_storages) = getuser(*user_id.get(),graphql_url_c.get().as_str()).await;

            storages.set(_storages);
            edit_storage.set(false);
            
        })
    };

    view!{cx,
        div(){

            h4{"Hi " (user.get().name)}
            
            article(){
                header(){"上传Lut"}
                fieldset(role="group"){
                    input(ref=upfile_ref,type="file",id="file",name="file")
        
                    button(aria-busy=*loading.get(),on:click = move|_|{
                        spawn_local_scoped(cx, async move {
                            loading.set(true);
                            let up_url = format!("{}/api/uplut",base_url_c.get());
                            // let up_url = "http://127.0.0.1:8081/api/uplut";
                            let filelist = upfile_ref
                            .get::<DomNode>()
                            .unchecked_into::<HtmlInputElement>().files().unwrap();
                            let file = filelist.item(0).unwrap();
                            let file_name = file.name();
                            // log::info!("{:?},{:?},{:?}",file.name(),file.size(),file.type_());
                            let file_array = sycamore::futures::JsFuture::from(file.array_buffer()).await.unwrap();
                            let file_bytes:Vec<u8> = web_sys::js_sys::Uint8Array::new(&file_array).to_vec().into();
                            let file_part = reqwest::multipart::Part::bytes(file_bytes).file_name(file_name.clone());
                            let form = reqwest::multipart::Form::new().part("file",file_part);
                            let client = reqwest::Client::new();
                            client.post(up_url)
                                .multipart(form)
                                .send()
                                .await
                                .expect("Failed to send request");
                            // images.set(getrawfiles(*user_id.get(),graphql_url_c.get().as_str()).await);
                            // log::info!("{:?}",images);
                            // loading.set(false);
        
                    })}){"submit"}
                }
        
            }

            article(){
                header(){"Raw 转换默认参数设定"}
                div(class="grid"){
                fieldset(){
                    legend(){"白平衡设置"}
                input(ref=auto_wb_ref,type="checkbox",role="switch",checked=user.get().wb,on:click = move |_| {
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

                fieldset(){
                legend(){"转换尺寸"}
                input(ref=auto_hf_ref,type="checkbox",role="switch",checked=user.get().half_size,on:click = move |_| {
                    let hf = auto_hf_ref.get::<DomNode>().unchecked_into::<HtmlInputElement>().checked();
                    if hf{
                        hf_label.set("半尺寸".to_string());
                    }
                    else{
                        hf_label.set("原尺寸".to_string());
                    }
                })
                label(){(hf_label.get())}
            }
        }
  
        div(class="grid"){
            
                fieldset(){
                legend(){"默认 Lut"}
                select(ref=lut_ref,aria-label="选择Lut"){
                    option(selected=true,value=-1){"不使用 Lut"}
                    Indexed(
                        iterable=luts,
                        view=move |cx, x|
                        view! {cx,
                            option(value = x.0,selected = user.get().lut_id == x.0 as i64){(x.1)}
                            },
                        )
                    }
                }
                fieldset(){
                legend(){"转换质量"}
                    fieldset(class="grid"){ 
                    input(bind:value=quality,type="range",min="10",max="100",step="1")
                    label(){(quality.get())}
                    }
                }
                }
                footer(style="display: flex;justify-content: center;align-items: center;"){
                button(on:click = bat){"保存"}
                }
                
            }
            article(){
                header(){"存储点设置"}
                table(){
                    thead(){
                        tr(){
                        th(scope="col"){"id"}
                        th(scope="col"){"存储点名称"}
                        th(scope="col"){"存储路径"}
                        th(scope="col"){"存储类型"}
                        }
                    }
                    tbody(){
                        Indexed(
                            iterable=storages,
                            view=|cx, x|
                            view! {cx,
                                tr(){
                                    th(scope="row"){(x.1.id)}
                                    td(){p(){(x.1.storage_name)}}
                                    td(){(x.1.storage_path)}
                                    td(){(x.1.storage_usage)}
                                }
                                },
                            )
                    }
                }
                footer(style="display: flex;justify-content: center;align-items: center;"){
                    button(on:click = move |_| edit_storage.set(true)){"新增"}
                    button(on:click = move |_| {
                        spawn_local_scoped(cx, async move {scan_files(*user_id.get(),base_url_c.get().as_str()).await;})}){"扫描"}
                    }
            }
            

            dialog(open=*edit_storage.get()){
                article(style="width: 100%; max-width: 80%"){
                    header(){
                        button(rel="prev",on:click= move |_| edit_storage.set(false)){}
                        p(){"自定义转换"}
                    }
                legend(){"存储点名称"}
                input(type = "text",bind:value=storage_name)

                legend(){"存储点路径"}
                input(type = "text",bind:value=storage_path)

                legend(){"存储点用途"}
                select(ref=storage_use_ref,aria-label="选择存储点用途"){
                    option(selected=true,value="source"){"原文件目录"}
                    option(value="cache"){"缓存目录"}
                    option(value="luts"){"Lut 目录"}
                    }
                    footer(style="display: flex;justify-content: center;align-items: center;"){
                        button(on:click= add_storage) { "保存" }
                        }
                    }
                }
            
            
             
        }
    }
}

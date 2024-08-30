use serde::{Deserialize, Serialize};
use sycamore::futures::spawn_local_scoped;
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
    log::info!("{:?}",response_body);
    let response_data: luts_query::ResponseData = response_body.data.expect("missing response data");
    response_data.luts.iter().map(|x| (format!("{}/{}",x.path.clone(),x.lut_name.clone()),x.lut_name.clone())).collect()
}

async fn getrawfiles(user_id:i32, url:&str) -> Vec<(usize, Image)> {
    // let base_url = web_sys::window().unwrap().location().origin().unwrap();
    // let url = format!("{}/api/graphql", base_url);
    // let url = format!("http://127.0.0.1:8081/api/graphql");
    
    let client = reqwest::Client::new();
    let variables = images_query::Variables {
        id: user_id.to_string(),
    };
    let response_body = 
        post_graphql::<ImagesQuery, _>(&client, url, variables).await.unwrap();
    log::info!("{:?}",response_body);
    let response_data: images_query::ResponseData = response_body.data.expect("missing response data");
    response_data.user.images.iter().enumerate().map(|(i,x)| {
        let _exif:Myexif = serde_json::from_str(&x.exif).unwrap();
        log::info!("{:?}",x.exif);
        
        (i,Image{
        id:x.id as i32,
        exif:_exif,
        filename: x.file_name.clone(),
        url: x.cached_url.clone(),
    })}).collect()
    // println!("{:?}",response_data.user.images[1].cache_file_name);

    // log::info!("{:?}",response_data.user.images);
}

async fn get_jpg(params: Parameters,base_url:&str) -> String {
    // let base_url = web_sys::window().unwrap().location().origin().unwrap();
    // let url = format!("http://127.0.0.1:8081/api/raw2jpg");
    log::info!("{:?}", params);
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

    let user_id = use_context::<RcSignal<i32>>(cx);

    images.set(getrawfiles(*user_id.get(),&graphql_url).await);

    

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


    // 处理图片点击事件
    let handle_image_click = move |index: usize| {
        current_index.set(index);
        is_zoomed.set(!*is_zoomed.get());
    };

    // 处理左切换按钮点击事件
    let handle_prev_click = move |_| {
        log::info!("{:?}", *current_index.get());
        current_index.set((*current_index.get() + images.get().len() - 1) % images.get().len());
    };

    // 处理右切换按钮点击事件
    let handle_next_click = move |_| {
        log::info!("{:?}", *current_index.get());
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
            let filename = images.get()[*current_index.get()].1.filename.clone();
            let image_id = images.get()[*current_index.get()].1.id.clone();
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
            // let file_name = images.get()[*current_index.get()].1.filename.clone();
            let image_id = images.get()[*current_index.get()].1.id.clone();
            save_jpg(url_file, image_id,base_url_c.get().as_str()).await;
            images.set(getrawfiles(*user_id.get(),graphql_url_c.get().as_str()).await);
            is_edit.set(false);
        })
    };

    
    // let luts: &Signal<Vec<(String,String)>> = create_signal(cx, Vec::new());
    view! {cx,
        


        div(class="custom-grid",hidden=*is_zoomed.get() || *is_edit.get()){
            Indexed(
                iterable=images,
                view=move |cx, (index,image)|
                view! {cx,
                        article(){

                                header(){
                                        small(on:click=move |_| {
                                            current_index.set(index);
                                            let _image = images.get()[index].1.clone();
                                            is_edit.set(true);
                                            file_name.set(_image.filename);
                                            img_url.set(_image.url);
                                        }){(image.filename)}
                                }
                                img(style="display: block;margin-left: auto;margin-right: auto;",src=image.url,on:click=move |_| handle_image_click(index))
                                footer(){
                                    small(){
                                        i(class="bx bx-aperture",style="margin-right: 20px;"){(image.exif.aperture)}
                                        i(class="bx bx-time-five",style="margin-right: 20px;"){((1.0/image.exif.shutter).round())}
                                        i(class="bx bx-album"){(image.exif.focal_len) " mm"}
                                        }
                                }
                            }
                    },
                )
            }

        (if *is_zoomed.get() {
                view! { cx,
                    div(class="image-container") {
                        img(src=images.get()[*current_index.get()].1.url,class="zoomed", on:click=move |_| is_zoomed.set(false))
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
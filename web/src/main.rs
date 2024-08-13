use std::string;

use serde::{Deserialize, Serialize};
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use web_sys::{HtmlInputElement, HtmlOptionElement};

#[derive(Serialize, Deserialize, Default, Debug, Clone,PartialEq)]
struct Parameters {
    filename: String,
    lut: String,
    wb: bool,
    exp_shift: f64,
    threshold: i32,
    url:String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Myexif {
    iso:i32,
    aperture:f32,
    shutter:f32,
    focal_len:i32,
    filename:String,
    url:String,
}


async fn getluts() -> Vec<(String, String)> {
    let base_url = web_sys::window().unwrap().location().origin().unwrap();
    let url = format!("{}/luts", base_url);
    // let url = format!("http://127.0.0.1:8081/luts");
    let body: Vec<String> = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let mut luts = Vec::new();
    luts.push(("No Lut".to_string(), "No Lut".to_string()));
    for i in body {
        luts.push((i.clone(), i));
    }
    luts
}

async fn getrawfiles() -> Vec<(usize,Myexif)> {
    let base_url = web_sys::window().unwrap().location().origin().unwrap();
    let url = format!("{}/rawfiles", base_url);
    // let url = format!("http://127.0.0.1:8081/rawfiles");
    let body: Vec<Myexif> = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    // body
    let mut rawfiles = Vec::new();
    // rawfiles.push(("No Lut".to_string(),"No Lut".to_string()));
    for i in 0..body.len()  {
        rawfiles.push((i,body[i].clone()));
    }
    rawfiles
}

async fn get_jpg(params: Parameters) -> String {
    let base_url = web_sys::window().unwrap().location().origin().unwrap();
    // let url = format!("http://127.0.0.1:8081/raw2jpg");
    log::info!("{:?}", params);
    let url = format!("{}/raw2jpg", base_url);
    let body: String = reqwest::Client::new()
        .post(&url)
        .json(&params)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    format!("{}/{}", base_url, body)
}

async fn save_jpg(url_file:String,filename:String) {
    let base_url = web_sys::window().unwrap().location().origin().unwrap();
    // let url = format!("http://127.0.0.1:8081/raw2jpg");
    // log::info!("{:?}", params);
    let url = format!("{}/save", base_url);
    reqwest::Client::new()
        .post(&url)
        .json(&(url_file,filename))
        .send()
        .await
        .unwrap();
    // format!("{}/{}", base_url, body)
}

#[component]
async fn ParametersSet<G: Html>(cx: Scope<'_>) -> View<G> {
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
    // let raw_ref = create_node_ref(cx);

    let images = create_signal(cx,Vec::new());

    images.set(getrawfiles().await);


    let upfile_ref = create_node_ref(cx);

    let auto_wb_ref = create_node_ref(cx);
    let wb_label = create_signal(cx, String::new());

    let loading = create_signal(cx, true);
    loading.set(true);

    wb_label.set("自动白平衡".to_string());


    // 当前显示的图片索引
    let current_index = create_signal(cx, 0);

    // 图片是否放大的状态
    let is_zoomed = create_signal(cx, false);

    let is_edit = create_signal(cx, false);


    // 处理图片点击事件
    let handle_image_click = move |index: usize| {
        current_index.set(index);
        is_zoomed.set(!*is_zoomed.get());
        
    };

    // 处理左切换按钮点击事件
    let handle_prev_click = move |_| {
        log::info!("{:?}",*current_index.get());
        current_index.set((*current_index.get() + images.get().len() - 1) % images.get().len());
        
    };

    // 处理右切换按钮点击事件
    let handle_next_click = move |_| {
        log::info!("{:?}",*current_index.get());
        current_index.set((*current_index.get() + 1) % images.get().len());
    };

    let bat = move |_| {
        spawn_local_scoped(cx, async move {
            loading.set(false);
            let lut = lut_ref
                .get::<DomNode>()
                .unchecked_into::<HtmlOptionElement>()
                .value();
            // let filename = raw_ref
            //     .get::<DomNode>()
            //     .unchecked_into::<HtmlOptionElement>()
            //     .value();
            let filename = images.get()[*current_index.get()].1.filename.clone();
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

            let exp_string_ = format!("lut: {} wb: {} exp_shift: {} threshold: {}",lut,if wb {"auto"}else{"camera"},if *exp_shift_flag.get() { "auto".to_string()} else {exp_shift.get().to_string()},if *threshold_flag.get() { "auto".to_string()} else {threshold.get().to_string()});
            let filename_ = filename.clone();
            img_url.set(
                get_jpg(Parameters {
                    filename,
                    lut,
                    wb,
                    exp_shift: exp_shift_,
                    threshold: threshold_,
                    url:"".to_string(),
                })
                .await,
            );
            file_name.set(filename_);
            exp_string.set(exp_string_);
            loading.set(true);
        })
    };

    let save = move |_|{
        spawn_local_scoped(cx, async move {
            let url_file = img_url.get().to_string().split("/").last().unwrap().to_string();
            let file_name = images.get()[*current_index.get()].1.filename.clone();
            save_jpg(url_file,file_name).await;
            images.set(getrawfiles().await);
            is_edit.set(false);
        })
    };

    let luts = create_signal(cx, getluts().await);
    // let raws = create_signal(cx, Vec::new());
    // raws.set(getrawfiles().await);

    view! {cx,
        progress(hidden=*loading.get()){}
        div(class="grid",hidden=*is_zoomed.get()){
            fieldset(role="group"){
                input(ref=upfile_ref,type="file",id="file",name="file")
                
                button(on:click = move|_|{
                    spawn_local_scoped(cx, async move {
                        let up_url = format!("{}/upfile",web_sys::window().unwrap().location().origin().unwrap());
                        // let up_url = "http://127.0.0.1:8081/upfile";
                        let filelist = upfile_ref
                        .get::<DomNode>()
                        .unchecked_into::<HtmlInputElement>().files().unwrap();
                        let file = filelist.item(0).unwrap();
                        let file_name = file.name();
                        log::info!("{:?},{:?},{:?}",file.name(),file.size(),file.type_());
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
                        images.set(getrawfiles().await);
                        log::info!("{:?}",images);

                })}){"submit"}
            }

            // select(ref=raw_ref,aria-label="选择Raw文件",width="20%"){
            //     option(selected=true,disabled=true){"选择Raw文件"}
                
            //     Indexed(
            //         iterable=raws,
            //         view=|cx, x|
            //         view! {cx,
            //             option(value = x.0){(x.1)}
            //             },
            //         )
            //     }
            
        }
        

        div(class="custom-grid",hidden=*is_zoomed.get() || *is_edit.get()){
            Indexed(
                iterable=images,
                view=move |cx, (index,image)|
                view! {cx,
                        // div(){
                            article(){
                                
                                header(){
                                    // fieldset(role="group"){
                                        small(on:click=move |_| {
                                            current_index.set(index);
                                            let _image = images.get()[index].1.clone();
                                            is_edit.set(true);
                                            file_name.set(_image.filename);
                                            img_url.set(_image.url);
                                        }){(image.filename)}
                                        // button(class="button2",on:click=move |_| {current_index.set(index);is_edit.set(true)}){"edit"}
                                    // }
                                }
                                img(src=image.url,on:click=move |_| handle_image_click(index))
                                // footer(){small(){i(){(format!("f: {} s: 1/{} {} mm",image.aperture,(1.0/image.shutter).round(),image.focal_len))}}}
                                footer(){
                                    small(){
                                        i(class="bx bx-aperture",style="margin-right: 20px;"){(image.aperture)}
                                        i(class="bx bx-time-five",style="margin-right: 20px;"){((1.0/image.shutter).round())}
                                        i(class="bx bx-album"){(image.focal_len) " mm"}
                                        }
                                }
                            }
                            // }
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
                    // img(src = "http://10.1.60.105:8081/tmp/57628e4c0dbf06268c38.jpg",onMouseOver="this.src='http://10.1.60.105:8081/tmp/79a88d82c894641efef9.jpg'",onMouseOut="this.src='http://10.1.60.105:8081/tmp/57628e4c0dbf06268c38.jpg'",style="hover{ cursor: pointer; }")
                    footer(){small(){i(){(exp_string.get())}}}
                }
                }
            div(){
                fieldset(class="grid"){
                
                
                // small(){"Lut选择"}
                // }

                // fieldset(){
                    // legend(){"白平衡"}
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
                    // small(){"白平衡"}
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
                    
                    // small(){"曝光补偿"}
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
            button(on:click= bat) { "处理" }
            button(on:click= save) { "保存" }
            }
        }
    }
    }
}
#[component]
async fn UpFile<G: Html>(cx: Scope<'_>) -> View<G> {
    let base_url = web_sys::window().unwrap().location().origin().unwrap();
    view! {cx,
        div{

            form(action=format!("{}/upfile",base_url),target="frameName",method="post",enctype="multipart/form-data"){
                fieldset(role="group",height="100px"){
                input(type="file",id="file",name="file")
                button(type="submit"){"submit"}
                }
            }
            iframe(src="",frameborder="0",name="frameName")
        }
    }
}

#[component]
fn Test2<G: Html>(cx: Scope) -> View<G> {
    // 图片列表
    let images = create_signal(cx,vec![
        ("0","http://127.0.0.1:8081/tmp/1722391821/96b550313037303733355f4e6779644c5277372e525732a57977313030c3cb4000000000000000ffa0.webp"),
        ("1","http://127.0.0.1:8081/tmp/1722391821/96ac50414e41383630312e525732a57977313030c3cb3ffccccccccccccdffa0.webp"),
        // ("2","https://via.placeholder.com/250"),
    ]);

    // 当前显示的图片索引
    let current_index = create_signal(cx, 0);

    // 图片是否放大的状态
    let is_zoomed = create_signal(cx, false);

    // 处理图片点击事件
    let handle_image_click = move |index: &str| {
        current_index.set(index.parse::<usize>().unwrap());
        is_zoomed.set(!*is_zoomed.get());
        
    };

    // 处理左切换按钮点击事件
    let handle_prev_click = move |_| {
        log::info!("{:?}",*current_index.get());
        current_index.set((*current_index.get() + images.get().len() - 1) % images.get().len());
        
    };

    // 处理右切换按钮点击事件
    let handle_next_click = move |_| {
        log::info!("{:?}",*current_index.get());
        current_index.set((*current_index.get() + 1) % images.get().len());
    };

    view! { cx,
        div() {
            div(class="custom-grid",hidden=*is_zoomed.get()){
            Indexed(
                iterable=images,
                view=move |cx, (index,image)|
                view! {cx,
                        // div(){
                            article(){
                                
                                header(){"aaa"}
                                img(src=image,on:click=move |_| handle_image_click(index))
                                // img(src = "http://10.1.60.105:8081/tmp/57628e4c0dbf06268c38.jpg",onMouseOver="this.src='http://10.1.60.105:8081/tmp/79a88d82c894641efef9.jpg'",onMouseOut="this.src='http://10.1.60.105:8081/tmp/57628e4c0dbf06268c38.jpg'",style="hover{ cursor: pointer; }")
                                footer(){
                                    // span(class="material-icons-outlined"){}
                                    // box-icon(name='aperture'){}
                                    small(){
                                    i(class="bx bx-aperture",style="margin-right: 20px;"){"2.0"}
                                    i(class="bx bx-time-five",style="margin-right: 20px;"){"1/200 "}
                                    i(class="bx bx-album"){"50mm"}
                                    }
                                    // small(){i(){"bbb"}}
                                }
                            }
                            // }
                    },
                )
            }
                (if *is_zoomed.get() {
                    view! { cx,
                        div(class="image-container") {
                            img(src=images.get()[*current_index.get()].1,class="zoomed", on:click=move |_| is_zoomed.set(false))
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
            // }
            // (if *is_zoomed.get() {
            //     view! { cx,
            //         div(class="image-container") {
            //             img(src=images.get()[*current_index.get()],class="zoomed", on:click=handle_image_click)
            //             div(class="button-container"){
            //             button(class="prev-button", on:click=handle_prev_click) { "Prev" }
            //             button(class="next-button", on:click=handle_next_click) { "Next" }
            //             }
            //         }
            //     }
            // } else {
            //     view! { cx,
            //         article(){
            //         img(src=images.get()[*current_index.get()], on:click=handle_image_click)
            //         }
            //     }
            // })
        }
    }
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    view! {cx,
        main(class="container"){
        div{
            // Test2()
            // Suspense(fallback=view! { cx, "Loading..." }) {
            //     Th2 {}
            // }
            // UpFile()
            ParametersSet()
        }
    }
    }
}

fn main() {
    console_log::init_with_level(log::Level::Debug).unwrap();
    sycamore::render(|cx| view! { cx, App {} });
}

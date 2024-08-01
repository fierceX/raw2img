use serde::{Deserialize, Serialize};
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use web_sys::{HtmlInputElement, HtmlOptionElement};

#[derive(Serialize, Deserialize, Default, Debug)]
struct Parameters {
    filename: String,
    lut: String,
    wb: bool,
    exp_shift: f64,
    threshold: i32,
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

async fn getrawfiles() -> Vec<(String, String)> {
    let base_url = web_sys::window().unwrap().location().origin().unwrap();
    let url = format!("{}/rawfiles", base_url);
    // let url = format!("http://127.0.0.1:8081/rawfiles");
    let body: Vec<String> = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let mut rawfiles = Vec::new();
    // rawfiles.push(("No Lut".to_string(),"No Lut".to_string()));
    for i in body {
        rawfiles.push((i.clone(), i));
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
    let raw_ref = create_node_ref(cx);

    let upfile_ref = create_node_ref(cx);

    let auto_wb_ref = create_node_ref(cx);
    let wb_label = create_signal(cx, String::new());
    wb_label.set("自动白平衡".to_string());

    let bat = move |_| {
        spawn_local_scoped(cx, async move {
            let lut = lut_ref
                .get::<DomNode>()
                .unchecked_into::<HtmlOptionElement>()
                .value();
            let filename = raw_ref
                .get::<DomNode>()
                .unchecked_into::<HtmlOptionElement>()
                .value();
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
                })
                .await,
            );
            file_name.set(filename_);
            exp_string.set(exp_string_);
        })
    };

    let luts = create_signal(cx, getluts().await);
    let raws = create_signal(cx, Vec::new());
    raws.set(getrawfiles().await);

    view! {cx,
        div(class="grid"){
            

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
                        raws.set(getrawfiles().await);
                        log::info!("{:?}",raws);

                })}){"submit"}
            }

            select(ref=raw_ref,aria-label="选择Raw文件",width="20%"){
                option(selected=true,disabled=true){"选择Raw文件"}
                
                Indexed(
                    iterable=raws,
                    view=|cx, x|
                    view! {cx,
                        option(value = x.0){(x.1)}
                        },
                    )
                }
            
        }
        div(class="grid") {
            div(style="display: flex;justify-content: center;align-items: center;"){
                article(){
                    header(){(file_name.get())}
                    img(src = img_url.get())
                    // img(src = "http://10.1.60.105:8081/tmp/57628e4c0dbf06268c38.jpg",onMouseOver="this.src='http://10.1.60.105:8081/tmp/79a88d82c894641efef9.jpg'",onMouseOut="this.src='http://10.1.60.105:8081/tmp/57628e4c0dbf06268c38.jpg'",style="hover{ cursor: pointer; }")
                    footer(){small(){i(){(exp_string.get())}}}
                }
                }
            article(){
                fieldset(class="grid"){
                
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
                // small(){"Lut选择"}
                // }

                // fieldset(){
                    // legend(){"白平衡"}
                    article(){
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
                footer(style="display: flex;justify-content: center;align-items: center;"){
                button(on:click= bat) { "处理" }
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
fn App<G: Html>(cx: Scope) -> View<G> {
    view! {cx,
        main(class="container"){
        div{
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

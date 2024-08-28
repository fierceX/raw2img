use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sycamore::{futures::spawn_local_scoped, web::html::nav};
use sycamore::prelude::*;
use sycamore_router::{navigate, HistoryIntegration, Route, Router, RouterProps};

use graphql_client::{reqwest::post_graphql, GraphQLQuery};

mod pages;

use pages::{home, login};

// #[derive(Clone, Copy, PartialEq, Eq)]
// struct UserId(i32);

// impl UserId {
//     fn is_enabled(self) -> bool {
//         self.0
//     }
// }




#[derive(Route)]
enum AppRoutes {
    #[to("/")]
    Home,
    #[to("/login")]
    Login,
    #[not_found]
    NotFound,
}


async fn check_auth() -> (bool,i32){
    // let base_url = web_sys::window().unwrap().location().origin().unwrap();
    // let url = "/api/check_auth";
    let url = format!("{}/api/check_auth",web_sys::window().unwrap().location().origin().unwrap());
    let res = reqwest::Client::new().post(url)
    .send().await.unwrap();
    if res.status() == StatusCode::OK{
        let user_id:String = res.json().await.unwrap();
        (true,user_id.parse().unwrap())
    }
    else{
        (false,-1)
    }
}

fn Header<G: Html>(cx: Scope) -> View<G>{
    view!{cx,
    header(class="container"){
      hgroup{
        h1{"Raw2Img"}
        p{"Raw格式文件转换Img工具"}
      }
    }
    }
}


async fn test_g() {
    // let client = reqwest::Client::new();
    // let variables = images_query::Variables {
    //     id: "1".to_string(),
    // };
    // let response_body = 
    //     post_graphql::<ImagesQuery, _>(&client, "http://127.0.0.1:8081/api/graphql", variables).await.unwrap();
    // // println!("{:?}",response_body);
    // let response_data: images_query::ResponseData = response_body.data.expect("missing response data");
    // // println!("{:?}",response_data.user.images[1].cache_file_name);

    // log::info!("{:?}",response_data.user.images);
    
}



#[component]
async fn App<G: Html>(cx: Scope<'_>) -> View<G> {
    let is_auth = create_signal(cx, true);
    let user_id = create_rc_signal(-1i32);
    
    
    // is_auth.set(check_auth().await);
    let (_is_auth,_user_id) = check_auth().await;
    
    user_id.set(_user_id);
    is_auth.set(_is_auth);
    log::info!("{} {}",is_auth,user_id);
    provide_context(cx,user_id);
    view! {cx,
        Router(
            integration=HistoryIntegration::new(),
            view=move |cx, route: &ReadSignal<AppRoutes>| {
                if *is_auth.get(){
                    view!{cx,
                        (match route.get().as_ref() {
                            AppRoutes::Home => view! { cx,
                                Header()
                                main(class="container"){
                                    div{
                                    home::Body()
                                    }
                                }
                            },
                            AppRoutes::Login => view! { cx,
                                Header()
                                main(class="container"){
                                    div{
                                login::login()
                                }}
                            },
                            AppRoutes::NotFound => view! { cx,
                                "404 Not Found"
                            },
                        })
                    }
                }
                else{
                    view!{cx,main(class="container"){
                        Header()
                        div{
                        login::login()
                        }}}
                }
            }
        )
    }
}

// #[component]
// fn App<G: Html>(cx: Scope) -> View<G> {
//     view! {cx,
//         main(class="container"){
//         div{
//             // Suspense(fallback=view! { cx, "Loading..." }) {
//             //     Body()
//             // } 
//             Body()
//         }
//     }
//     }
// }

fn main() {
    
    console_log::init_with_level(log::Level::Debug).unwrap();
    sycamore::render(|cx| view! { cx, App {} });
}

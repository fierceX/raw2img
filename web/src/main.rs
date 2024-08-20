use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sycamore::{futures::spawn_local_scoped, web::html::nav};
use sycamore::prelude::*;
use sycamore_router::{navigate, HistoryIntegration, Route, Router, RouterProps};


mod pages;

use pages::{home, login};

#[derive(Route)]
enum AppRoutes {
    #[to("/")]
    Home,
    #[to("/login")]
    Login,
    #[not_found]
    NotFound,
}


async fn check_auth() -> bool{
    let res = reqwest::Client::new().post("http://127.0.0.1:8081/api/check_auth")
    .send().await.unwrap();
    if res.status() == StatusCode::OK{
        true
    }
    else{
        false
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

#[component]
async fn App<G: Html>(cx: Scope<'_>) -> View<G> {
    let is_auth = create_signal(cx, false);
    is_auth.set(check_auth().await);
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

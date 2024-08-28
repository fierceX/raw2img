use reqwest::StatusCode;
use sycamore::{futures::spawn_local_scoped, prelude::*};
use sycamore_router::navigate;
use web_sys::window;

#[component]
pub fn login<G: Html>(cx: Scope) -> View<G> {
    let username = create_signal(cx, String::new());
    let password = create_signal(cx, String::new());
    view! { cx,
        div {
            input(type="text",id = "username",name="username",bind:value=username)
            input(type="password",id = "password",name="password",bind:value=password)
            button(type="submit",on:click=move|_|{
                spawn_local_scoped(cx, async move {
                // log::info!("u:{} p:{}",username,password);
                let url = format!("{}/auth",web_sys::window().unwrap().location().origin().unwrap());
                let res = reqwest::Client::new().post(url)
                .form(&[("username",username.get().to_string()),("password",password.get().to_string())])
                .send().await.unwrap();
                if res.status() == StatusCode::OK{
                    log::info!("ok");
                    // navigate("/")
                    web_sys::window().unwrap().location().reload();
                }
                else{
                    log::info!("error");
                }
                }
            )
            })
        }
    }
}

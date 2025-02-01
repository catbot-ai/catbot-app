#![allow(non_snake_case)]

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

static CSS: Asset = asset!("/assets/main.css");

pub fn App() -> Element {
    // let mut name = use_signal(String::new);
    // let mut greet_msg = use_signal(String::new);

    // let greet = move |_: FormEvent| async move {
    //     if name.read().is_empty() {
    //         return;
    //     }

    //     let name = name.read();
    //     let args = serde_wasm_bindgen::to_value(&GreetArgs { name: &name }).unwrap();
    //     // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    //     let new_msg = invoke("greet", args).await.as_string().unwrap();
    //     greet_msg.set(new_msg);
    // };

    rsx! {
        document::Stylesheet { href: CSS }

        iframe {
            src: "https://portfolio.jup.ag/",
            width: "100%",
            height: "600px",
        }
    }
}

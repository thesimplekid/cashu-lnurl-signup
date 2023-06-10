use gloo::console::__macro::{Array, JsValue};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/js/imp.js")]
extern "C" {
    #[wasm_bindgen]
    pub async fn get_pubkey() -> JsValue;
    #[wasm_bindgen]
    pub async fn encrypt_content(pubkey: String, content: String) -> JsValue;
    #[wasm_bindgen]
    pub async fn sign_event(
        created_at: i64,
        content: String,
        tags: Array,
        pubkey: String,
    ) -> JsValue;
}

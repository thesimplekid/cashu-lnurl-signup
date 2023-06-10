use std::str::FromStr;

use anyhow::Result;
use gloo::console::__macro::{Array, JsValue};
use log::{debug, warn};
use nostr_sdk::{secp256k1::XOnlyPublicKey, Client, Event, EventBuilder, Keys, Tag, Url};
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::{platform::spawn_local, prelude::*};

use crate::bindings::{self, encrypt_content, sign_event};

#[derive(Debug, Clone, Copy)]
pub enum State {
    Enterinfo,
    VerifyInfo,
    SignedUp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSignUp {
    /// Cashu mint
    pub mint: String,
    /// LN Address username
    pub username: String,
    /// Nostr Relays
    pub relays: Vec<String>,
}

#[derive(Clone)]
pub enum Msg {
    SetPubkey(String),
    GetNIP07Key,
    Next,
    Submit,
}

async fn get_pubkey() -> Result<Option<String>> {
    let key = unsafe { bindings::get_pubkey().await };
    Ok(key.as_string())
}

pub struct App {
    state: State,
    client: Option<Client>,
    lnurl_service_pubkey: XOnlyPublicKey,
    pubkey: Option<String>,
    pubkey_input: NodeRef,
    username: Option<String>,
    username_input: NodeRef,
    domain: AttrValue,
    mint: Option<String>,
    mint_input: NodeRef,
}
impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let lnurl_service_pubkey = XOnlyPublicKey::from_str(env!("LNURL_SERVICE_PUBKEY")).unwrap();

        Self {
            client: None,
            lnurl_service_pubkey,
            state: State::Enterinfo,
            domain: "saoirse.dev".into(),
            pubkey: None,
            pubkey_input: NodeRef::default(),
            username: None,
            username_input: NodeRef::default(),
            mint: None,
            mint_input: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        fn create_client(pubkey: &str) -> Client {
            let connect_relay = Url::from_str("wss://thesimplekid.space/").unwrap();
            let key = XOnlyPublicKey::from_str(&pubkey).unwrap();
            let keys = Keys::from_public_key(key);
            let client = Client::new(&keys);

            let client_clone = client.clone();
            spawn_local(async move {
                client_clone.add_relays(vec![connect_relay]).await.ok();
                client_clone.connect().await;
            });

            client
        }
        match msg {
            Msg::GetNIP07Key => {
                ctx.link().send_future(async {
                    match get_pubkey().await {
                        Ok(Some(pubkey)) => Msg::SetPubkey(pubkey),
                        _ => Msg::SetPubkey("".to_string()),
                    }
                });

                true
            }
            Msg::SetPubkey(pubkey) => {
                self.pubkey_input
                    .cast::<web_sys::HtmlInputElement>()
                    .expect("Failed to cast NodeRef to HtmlInputElement")
                    .set_value(&pubkey);

                self.client = Some(create_client(&pubkey));
                self.pubkey = Some(pubkey);
                debug!("{:?}", self.pubkey);
                true
            }
            Msg::Next => {
                if self.pubkey.is_none() {
                    let pubkey = self
                        .pubkey_input
                        .cast::<HtmlInputElement>()
                        .unwrap()
                        .value();
                    self.client = Some(create_client(&pubkey));
                    self.pubkey = Some(pubkey);
                }

                self.username = Some(
                    self.username_input
                        .cast::<HtmlInputElement>()
                        .unwrap()
                        .value(),
                );

                // TODO: Check mint is valid
                self.mint = Some(self.mint_input.cast::<HtmlInputElement>().unwrap().value());

                self.state = State::VerifyInfo;

                true
            }
            Msg::Submit => {
                let signup_msg = UserSignUp {
                    mint: self.mint.clone().unwrap(),
                    username: self.username.clone().unwrap(),
                    relays: vec![],
                };

                if let Some(client) = &self.client {
                    debug!("client");
                    let client_clone = client.clone();
                    let pubkey = client.keys().public_key();
                    let rec_pubkey = self.lnurl_service_pubkey.clone();

                    spawn_local(async move {
                        let cipher_text = unsafe {
                            encrypt_content(
                                rec_pubkey.to_string(),
                                serde_json::to_string(&signup_msg).unwrap(),
                            )
                            .await
                            .as_string()
                        };

                        let event = EventBuilder::new(
                            nostr_sdk::Kind::Custom(20420),
                            cipher_text.unwrap(),
                            &[Tag::PubKey(rec_pubkey, None)],
                        )
                        .to_unsigned_event(pubkey);

                        let tags = event
                            .tags
                            .into_iter()
                            .map(|t| {
                                t.as_vec()
                                    .iter()
                                    .map(|x| JsValue::from_str(x))
                                    .collect::<Array>()
                            })
                            .collect::<Array>();

                        let signed_event = unsafe {
                            sign_event(
                                event.created_at.as_i64(),
                                event.content,
                                tags,
                                event.pubkey.to_string(),
                            )
                            .await
                            .as_string()
                        };
                        if let Some(event) = signed_event {
                            let event: Event = serde_json::from_str(&event).unwrap();
                            debug!("sig: {:?}", event.as_json());
                            if let Err(err) = client_clone.send_event(event).await {
                                warn!("{err}");
                            }
                        }
                    });
                    self.state = State::SignedUp;
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let next = ctx.link().callback(|_| Msg::Next);
        let get_pubkey = ctx.link().callback(|_| Msg::GetNIP07Key);
        let submit = ctx.link().callback(|_| Msg::Submit);
        match self.state {
            State::Enterinfo => {
                html! {
                    <>
                        <h2 class="text-4xl font-extrabold dark:text-white">{ "LN Url Signup" }</h2>
                        <div class="flex items-center justify-center mb-6">
                            <div class="w-max">
                                <label for="default-input" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{"LN Address"}</label>
                                <div class="flex items-center">
                                    <input type="text" id="default-input" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" ref={self.username_input.clone()}/>
                                    <h2 class="ml-2 mb-2 text-lg font-semibold text-gray-900 dark:text-white"> { format!("@{}", self.domain.clone()) }</h2>
                                </div>
                            </div>
                       </div>

                    <div class="flex items-center justify-center mb-6">
                      <div class="w-max">
                        <label for="default-input" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{"Enter your public key"}</label>
                        <div class="flex items-center">
                          <input type="text" id="default-input" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" ref={self.pubkey_input.clone()}/>
                          <button type="button" class="focus:outline-none text-white bg-purple-700 hover:bg-purple-800 focus:ring-4 focus:ring-purple-300 font-medium rounded-lg text-sm px-5 py-2.5 mb-2 dark:bg-purple-600 dark:hover:bg-purple-700 dark:focus:ring-purple-900" onclick={get_pubkey}>{ "Get NIP07 Pubkey" } </button>
                        </div>
                      </div>
                    </div>

                    <div class="flex items-center justify-center mb-6">
                      <div class="w-max">
                        <label for="default-input" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{"Enter Mint"}</label>
                        <div class="flex items-center">
                          <input type="text" id="default-input" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" ref={self.mint_input.clone()}/>
                        </div>
                      </div>
                    </div>

                    <div class="flex items-center justify-center mb-6">
                        <button type="button" class="focus:outline-none text-white bg-purple-700 hover:bg-purple-800 focus:ring-4 focus:ring-purple-300 font-medium rounded-lg text-sm px-5 py-2.5 ml-4 dark:bg-purple-600 dark:hover:bg-purple-700 dark:focus:ring-purple-900" onclick={next}>{ "Next" }</button>
                    </div>
                </>
                }
            }
            State::VerifyInfo => {
                html! {
                        <>
                        <div class="flex items-center justify-center mb-6">
                    <h2 class="block mb-2 text-lg font-medium text-gray-900 dark:text-white">{"Payments Sent to "} <span class="font-bold">{format!("{}@{}", self.username.clone().unwrap(), self.domain.clone())}</span> <>{format!(" will be sent as cashu token from {} to ", self.mint.clone().unwrap())}</> <span class="font-bold">{self.pubkey.clone().unwrap()}</span></h2>

                    </div>
                    <div class="flex items-center justify-center mb-6">
                        <button type="button" class="focus:outline-none text-white bg-purple-700 hover:bg-purple-800 focus:ring-4 focus:ring-purple-300 font-medium rounded-lg text-sm px-5 py-2.5 ml-4 dark:bg-purple-600 dark:hover:bg-purple-700 dark:focus:ring-purple-900" onclick={submit}>{ "Submit" }</button>
                    </div>
                    </>

                }
            }
            State::SignedUp => {
                html! {
                <div class="flex items-center justify-center mb-6">
                    <h2 class="block mb-2 text-lg font-medium text-gray-900 dark:text-white">{ "Welcome! Check your DMs." }</h2>
                </div>
                }
            }
        }
    }
}

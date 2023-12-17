use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

use asset::Registry;
use gloo::console::{console, console_dbg};
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
mod asset;
mod asset_entry;

use web_sys::HtmlInputElement as InputElement;
use yew::events::{FocusEvent, KeyboardEvent};
use yew::html::Scope;
use yew::{classes, html, Classes, Component, Context, Html, NodeRef, TargetCast};

use crate::asset::{Asset, Filter};

/// The possible states a fetch request can be in.
pub enum FetchState<T> {
    NotFetching,
    Fetching,
    Success(T),
    Failed(),
}

enum Msg {
    SetMarkdownFetchState(FetchState<Vec<&'static Asset>>),
    GetAssets(Filter),
    GetMarkdown,
    GetError,
}
struct App {
    state: FetchState<Vec<&'static Asset>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            state: FetchState::NotFetching,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetMarkdownFetchState(fetch_state) => {
                console_dbg!("Complete");
                self.state = fetch_state;
                true
            }
            Msg::GetAssets(filter) => {
                //let box0 = Box::new(self.registry);
                //let box1: &'static mut Registry = Box::leak(self.registry);
                //let box2 = Box::leak(REGISTRY);
                //let mut reg1 = self.registry.lock().unwrap();
                console_dbg!("GetAssets");
                ctx.link().send_future(async move {
                    console_dbg!("GetAssets query");
                    match REGISTRY.query(filter).await {
                        Ok(md) => Msg::SetMarkdownFetchState(FetchState::Success(md)),
                        Err(err) => Msg::SetMarkdownFetchState(FetchState::Failed()),
                    }
                });
                ctx.link()
                    .send_message(Msg::SetMarkdownFetchState(FetchState::Fetching));
                false
            }
            Msg::GetMarkdown => false,
            Msg::GetError => {
                //ctx.link().send_future(async {
                //match fetch_markdown(INCORRECT_URL).await {
                //    Ok(md) => Msg::SetMarkdownFetchState(FetchState::Success(md)),
                //    Err(err) => Msg::SetMarkdownFetchState(FetchState::Failed(err)),
                //}
                // });
                ctx.link()
                    .send_message(Msg::SetMarkdownFetchState(FetchState::Fetching));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let body = match &self.state {
            FetchState::NotFetching => html! {"" },
            FetchState::Fetching => html! {"Fetching" },
            FetchState::Success(data) => render_list(data.into()),
            FetchState::Failed() => html! {"error"},
        };
        html! {
            <div>
                    <button onclick={ctx.link().callback(|_| Msg::GetAssets(Filter::All))}>
                        { "Get All" }
                    </button>
                    <button onclick={ctx.link().callback(|_| Msg::GetAssets(Filter::Main))}>
                        { "Get Main" }
                    </button>
                    <button onclick={ctx.link().callback(|_| Msg::GetAssets(Filter::Amp))}>
                        { "Get Amp" }
                    </button>
                    <button onclick={ctx.link().callback(|_| {
                        console_dbg!("Complete");
                        Msg::GetAssets(Filter::Stablecoins)
                    })}>
                        { "Get Stablecoins" }
                    </button>

                    { self.view_input(ctx.link()) }
                    <div> { body } </div>
            </div>
        }
    }
}
impl App {
    const fn is_alphanumeric(key_code: u32) -> bool {
        (key_code >= 48 && key_code <= 57)
            || (key_code >= 65 && key_code <= 90)
            || (key_code >= 97 && key_code <= 122)
    }

    fn view_input(&self, link: &Scope<Self>) -> Html {
        let onkeypress = link.batch_callback(|e: KeyboardEvent| {
            console_dbg!(e.key_code());
            if Self::is_alphanumeric(e.key_code()) {
                let input: InputElement = e.target_unchecked_into();
                let value = input.value();
                //input.set_value("");
                Some(Msg::GetAssets(Filter::Text(value)))
            } else {
                None
            }
        });
        html! {
            <input
                class="new-todo"
                placeholder="What needs to be done?"
                {onkeypress}
            />
        }
    }
}
use lazy_static::lazy_static;

lazy_static! {
    static ref REGISTRY: Registry = Registry::new();
}
impl Asset {
    fn render(&self) -> Html {
        let asset = self.asset_entry.as_ref();
        let name = asset.map_or("", |a| a.name.as_str());
        let ticker = asset.map_or("", |a| a.ticker.as_ref().map_or("", |t| t.as_str()));
        let base64 = self.icon.as_ref();
        let image = format!(
            "data:image/png;base64, {}",
            base64.unwrap_or(&"".to_string())
        );
        html! {
            <tr>
            <th> <img class="icon" src={image}/> </th>
            <th> { ticker } </th>
            <th> { name } </th>
            <th> { self.asset_id.to_string() } </th>
            </tr>
        }
    }
}

fn render_list(assets: &Vec<&Asset>) -> Html {
    html! {
        <table>
        { for assets.iter().map(|x| x.render()) }
        </table>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

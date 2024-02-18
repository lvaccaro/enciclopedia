use gloo::console::console_dbg;
use gloo::storage::LocalStorage;
use gloo_storage::Storage;
use std::borrow::Borrow;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlInputElement as InputElement;
use web_sys::{EventTarget, HtmlElement, MouseEvent};
use yew::events::KeyboardEvent;
use yew::html::Scope;
use yew::{html, Component, Context, Html, TargetCast};

mod asset;
mod asset_entry;
mod registry;

use crate::asset::Asset;
use crate::registry::{Filter, Registry};

/// The possible states a fetch request can be in.
pub enum FetchState {
    NotFetching,
    Fetching,
    Success(Vec<&'static Asset>),
    Single(Asset, Option<String>, Option<String>, bool),
    Failed(),
}

enum Msg {
    SetMarkdownFetchState(FetchState),
    GetVisibleAssets(),
    GetAssets(Filter),
    GetAsset(String),
    GetError,
}
struct App {
    state: FetchState,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        console_dbg!("create");
        ctx.link().send_message(Msg::GetAssets(Filter::Main));
        Self {
            state: FetchState::NotFetching,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetMarkdownFetchState(fetch_state) => {
                self.state = fetch_state;
                true
            }
            Msg::GetVisibleAssets() => {
                let ids: Vec<elements::AssetId> = LocalStorage::get("ids").unwrap();
                ctx.link().send_future(async move {
                    match REGISTRY.query_by_ids(ids).await {
                        Ok(ass) => Msg::SetMarkdownFetchState(FetchState::Success(ass)),
                        Err(_) => Msg::SetMarkdownFetchState(FetchState::Failed()),
                    }
                });
                ctx.link()
                    .send_message(Msg::SetMarkdownFetchState(FetchState::Fetching));
                false
            }
            Msg::GetAssets(filter) => {
                console_dbg!("update");
                ctx.link().send_future(async move {
                    match REGISTRY.query(filter).await {
                        Ok(ids) => {
                            console_dbg!("query await");
                            LocalStorage::set("ids", ids.clone()).unwrap();
                            match REGISTRY.query_by_ids(ids).await {
                                Ok(ass) => {
                                    console_dbg!("query_by_ids await");
                                    Msg::SetMarkdownFetchState(FetchState::Success(ass))
                                }
                                Err(_) => Msg::SetMarkdownFetchState(FetchState::Failed()),
                            }
                        }
                        Err(_) => Msg::SetMarkdownFetchState(FetchState::Failed()),
                    }
                });
                ctx.link()
                    .send_message(Msg::SetMarkdownFetchState(FetchState::Fetching));
                false
            }
            Msg::GetAsset(id) => {
                ctx.link().send_future(async move {
                    let id = elements::AssetId::from_str(&id).unwrap();
                    match REGISTRY.query_by_id(id).await {
                        Ok(ass) => {
                            let supply: Option<String> = REGISTRY.supply(id).await.ok();
                            let price: Option<String> = REGISTRY.price(id).await.ok();
                            let verified = ass
                                .asset_entry
                                .as_ref()
                                .is_some_and(|x| x.verifies().is_ok_and(|x| x));
                            Msg::SetMarkdownFetchState(FetchState::Single(
                                ass.clone(),
                                supply,
                                price,
                                verified,
                            ))
                        }
                        Err(err) => Msg::SetMarkdownFetchState(FetchState::Failed()),
                    }
                });
                ctx.link()
                    .send_message(Msg::SetMarkdownFetchState(FetchState::Fetching));
                false
            }
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
            FetchState::Success(data) => self.view_list(ctx, data.into()),
            FetchState::Single(asset, supply, price, verified) => self.view_dialog(
                ctx,
                asset.clone(),
                supply.clone(),
                price.clone(),
                verified.clone(),
            ),
            FetchState::Failed() => html! {"error"},
        };
        console_dbg!("body");
        html! {
            <div id="nescss">
                { self.view_header(ctx)}
                <div class="container">
                <main class="main-content">

                { self.view_filters(ctx)}

                <section class="topic">
                <section class="showcase">
                { body }
                </section>
                </section>

                </main>
                </div>
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

    fn view_header(&self, ctx: &Context<Self>) -> Html {
        let github_link = "https://github.com/lvaccaro/enciclopedia";
        html! {
            <header class="sticky">
                <div class="container">
                    <div class="nav-brand">
                        <a href="#"><h1><i class="snes-jp-logo brand-logo"></i> { "Enciclopedia" } </h1></a>
                        <p> { "Liquid asset registry" } </p>
                    </div>
                    <div class="social-buttons">
                        <div class="share">
                            <a href={ github_link } target="_blank"><i class="nes-icon github"></i></a>
                        </div>
                    </div>
                </div>
                </header>
        }
    }

    fn view_filters(&self, ctx: &Context<Self>) -> Html {
        html! {
            <section class="topic">
                <section class="showcase">
                    <section class=""><h3 class="title"> </h3>

                    <div class="item">
                            <button class="nes-btn is-primary" onclick={ctx.link().callback(|_| Msg::GetAssets(Filter::Main))}>
                                { "Main" }
                            </button> { " " }
                            <button class="nes-btn is-success" onclick={ctx.link().callback(|_| Msg::GetAssets(Filter::Amp))}>
                                { "Amp" }
                            </button> { " " }
                            <button class="nes-btn is-warning" onclick={ctx.link().callback(|_| Msg::GetAssets(Filter::Stablecoins))}>
                                { "Stablecoins" }
                            </button> { " " }
                            <button class="nes-btn is-error" onclick={ctx.link().callback(|_| Msg::GetAssets(Filter::Iconed))}>
                                { "Iconed" }
                            </button> { " " }
                            <button class="nes-btn" onclick={ctx.link().callback(|_| Msg::GetAssets(Filter::All))}>
                                { "All" }
                            </button>
                    </div>
                    </section>
                </section>
                </section>
        }
    }

    fn view_input(&self, ctx: &Scope<Self>) -> Html {
        let onkeypress = ctx.batch_callback(|e: KeyboardEvent| {
            console_dbg!(e.key_code());
            if Self::is_alphanumeric(e.key_code()) {
                let input: InputElement = e.target_unchecked_into();
                let value = input.value();
                Some(Msg::GetAssets(Filter::Text(value)))
            } else {
                None
            }
        });
        html! {
            <div class="nes-field">
            <label for="name_field"> { "Search by" } </label>
            <input type="text" id="name_field" class="nes-input"
            {onkeypress}
            />
            </div>
        }
    }

    fn view_list(&self, ctx: &Context<Self>, assets: &Vec<&Asset>) -> Html {
        html! {

            <section class="topic">
            <table class="nes-table nes-table-responsive is-bordered">
            <tbody>
            { for assets.iter().map(|x| self.view_item(ctx, x)) }
            </tbody>
            </table>
            </section>
        }
    }

    fn view_dialog(
        &self,
        ctx: &Context<Self>,
        asset: Asset,
        supply: Option<String>,
        price: Option<String>,
        verified: bool,
    ) -> Html {
        let onkeypress_cancel = ctx
            .link()
            .batch_callback(|_: MouseEvent| Some(Msg::GetVisibleAssets()));
        let onkeypress_validate = ctx
            .link()
            .batch_callback(|_: MouseEvent| Some(Msg::GetVisibleAssets()));
        let asset_entry = asset.asset_entry.as_ref();
        let name = asset_entry.map_or("", |a| a.name.as_str());
        let ticker = asset_entry.map_or("", |a| a.ticker.as_ref().map_or("", |t| t.as_str()));
        let domain = asset_entry.map_or("", |a| a.domain().map_or("", |d| d));
        let esplora = format!(
            "https://blockstream.info/liquid/asset/{}",
            asset.asset_id.to_string()
        );
        let sideswap = format!("https://sideswap.io/swap-market/?product={}", ticker);
        let base64 = asset.icon.as_ref();
        let image = format!(
            "data:image/png;base64, {}",
            base64.unwrap_or(&"".to_string())
        );
        let pair = asset.metadata.as_ref().and_then(|x| x.pair.clone());
        html! {

            <div>
            <section class="nes-container is-dark member-card">
                <div class="avatar">
                    <img src={image} class=""/>
                    <span class="nes-text is-error" hidden={ verified }>{ "unverified" }</span>
                    <span class="nes-text is-primary" hidden={ !verified }>{ "verified" }</span>
                </div>
                <div class="profile">
                    <h4 class="name">{ticker}</h4>
                    <p>{ name } { " by " } { domain }</p>
                </div>
            </section>

            <section class="nes-container  with-title topic">
                    <p class="title"> { "Asset ID" } </p>
                    <span style="overflow-wrap: break-word;"> { asset.asset_id.to_string() }</span>
            </section>

            <section class="topic">
            <div class="nes-field is-inline">
                <label for="warning_field"> { "Circulating amount" } </label>
                <input type="text" class="nes-input is-warning" value={ supply.unwrap_or("".to_string()) }/>
            </div>
            <div class="nes-field is-inline" hidden={ price.is_none() }>
                <label for="warning_field"> { "Price " } { pair.unwrap_or("".to_string()) }</label>
                <input type="text" class="nes-input is-success" value={ price.unwrap_or("".to_string()) }/>
            </div>
            </section>

            <section class="topic">
                <div>
                    <a class="nes-badge" href="#" hidden={!asset.is_amp()}>
                        <span class="is-success" > { "amp" } </span>
                    </a>
                    { " " }
                    <a class="nes-badge" href="#" hidden={!asset.is_stablecoin()}>
                        <span class="is-warning"> { "stablecoin" } </span>
                    </a>
                    { " " }
                    <a class="nes-badge" href="#" hidden={!asset.is_meme()}>
                        <span class="is-error"> { "meme" } </span>
                    </a>
                </div>
            </section>

            <section class="topic">
                <menu class="dialog-menu">
                    <button class="nes-btn" onclick={onkeypress_cancel}>{"Back"}</button>
                    { " " }
                    <a class="nes-btn is-primary" href={ esplora }>{ "Esplora" }</a>
                    { " " }
                    <a class="nes-btn is-primary" href={ sideswap }>{ "Sideswap" }</a>
                    { " " }
                    //<button class="nes-btn is-success" onclick={onkeypress_validate}>{"Validate"}</button>
                </menu>
            </section>
            </div>
        }
    }
    fn view_item(&self, ctx: &Context<Self>, asset: &Asset) -> Html {
        let asset_entry = asset.asset_entry.as_ref();
        let name = asset_entry.map_or("", |a| a.name.as_str());
        let ticker = asset_entry.map_or("", |a| a.ticker.as_ref().map_or("", |t| t.as_str()));
        let base64 = asset.icon.as_ref();
        let image = format!(
            "data:image/png;base64, {}",
            base64.unwrap_or(&"".to_string())
        );
        let onkeypress = ctx.link().batch_callback(move |e: MouseEvent| {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlElement>().ok());

            let id = input.unwrap().id();
            Some(Msg::GetAsset(id))
        });
        html! {
            <tr>
            <th> <img src={image} class="nes-icon coin is-large"/> </th>
            <th> { ticker } </th>
            <th> { name } </th>
            <th>
                <a class="nes-badge" href="#" hidden={!asset.is_amp()}>
                    <span class="is-success" > { "amp" } </span>
                </a>
                <a class="nes-badge" href="#" hidden={!asset.is_stablecoin()}>
                    <span class="is-warning"> { "stablecoin" } </span>
                </a>
                <a class="nes-badge" href="#" hidden={!asset.is_meme()}>
                    <span class="is-error"> { "meme" } </span>
                </a>
            </th>
            <th>
                <button type="button" class="nes-btn is-primary" onclick={ onkeypress } id={ asset.asset_id.to_string() }>//tx.link().callback(|_| Msg::GetAsset(asset.clone()))}>
                { "<>" }
                </button>
            </th>
            //<th> { self.asset_id.to_string() } </th>
            </tr>

        }
    }
}
use lazy_static::lazy_static;

lazy_static! {
    static ref REGISTRY: Registry = Registry::new();
}

fn main() {
    yew::Renderer::<App>::new().render();
}

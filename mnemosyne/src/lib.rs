#![recursion_limit = "1024"]

use lazy_static::lazy_static;
use log::Level;
use reqwest::Url;
use wasm_bindgen::prelude::*;
use yew::utils::document;
use yew::{html, App, Component, ComponentLink, Html};

use components::app_contents::AppContents;
use components::menu::Menu;

use crate::components::NotificationBar;

mod cdl_objects;
mod components;
mod context_bus;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

lazy_static! {
    pub static ref GRAPHQL_URL: Url = Url::parse("http://localhost:50106/graphql").unwrap();
}

pub struct Model;

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self
    }

    fn update(&mut self, _: Self::Message) -> bool {
        false
    }

    fn change(&mut self, _: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
            <nav>
                <div class="nav-container"><Menu /></div>
            </nav>
            <main>
                <div class="container">
                    <div class="row">
                        <div class="col col-sm-12 col-xlg-8"><AppContents /></div>
                        <div class="col col-sm-12 col-xlg-4"><NotificationBar /></div>
                    </div>
                </div>
            </main>
            </>
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    console_log::init_with_level(Level::Debug).expect("Init logger");
    yew::initialize();

    let content = document().get_element_by_id("content").unwrap();

    App::<Model>::new().mount(content);
}

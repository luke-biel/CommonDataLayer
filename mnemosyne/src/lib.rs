#![recursion_limit = "1024"]

use lazy_static::lazy_static;
use log::Level;
use reqwest::Url;
use wasm_bindgen::prelude::*;
use yew::utils::document;
use yew::{html, App, Component, ComponentLink, Html, Threaded};
use yew_router::prelude::*;

use crate::app_contents::AppContents;
use crate::context_bus::ContextBus;
use crate::menu::Menu;

mod app_contents;
mod cdl_objects;
mod components;
mod context_bus;
pub mod menu;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

lazy_static! {
    pub static ref GRAPHQL_URL: Url = Url::parse("http://localhost:50106/graphql").unwrap();
}

pub struct Model {
    link: ComponentLink<Self>,
}

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Model { link }
    }

    fn update(&mut self, _: Self::Message) -> bool {
        false
    }

    fn change(&mut self, _: Self::Properties) -> bool {
        false
    }

    //noinspection RsTypeCheck
    fn view(&self) -> Html {
        html! {
            <>
                <Menu />
                <AppContents />
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

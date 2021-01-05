#![recursion_limit = "1024"]

mod cdl_objects;
mod graphql;
mod schema_registry;

use crate::cdl_objects::CDLSchemas;
use crate::schema_registry::{SchemaRegistry, SchemaRegistryProps};
use reqwest::Url;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use yew::utils::document;
use yew::{html, run_loop, App, Component, ComponentLink, Html};
use yewtil::future::LinkFuture;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

enum SubPage {
    Fetching,
    Error(String),
    SchemaRegistry(Rc<CDLSchemas>),
}

struct Model {
    link: ComponentLink<Self>,
    page: SubPage,
}

enum Msg {
    SetFetchState(FetchState),
}

enum FetchState {
    Schemas(CDLSchemas),
    Failed(String),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        link.send_future(async {
            match CDLSchemas::fetch(Url::parse("http://localhost:50106/graphql").unwrap()).await {
                Ok(schemas) => Msg::SetFetchState(FetchState::Schemas(schemas)),
                Err(err) => Msg::SetFetchState(FetchState::Failed(err)),
            }
        });

        Model {
            link,
            page: SubPage::Fetching,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::SetFetchState(fetch_state) => match fetch_state {
                FetchState::Schemas(items) => self.page = SubPage::SchemaRegistry(Rc::new(items)),
                FetchState::Failed(error) => self.page = SubPage::Error(error),
            },
        }

        true
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    //noinspection RsTypeCheck
    fn view(&self) -> Html {
        match &self.page {
            SubPage::Fetching => html! { "Fetching" },
            SubPage::SchemaRegistry(schemas) => {
                let schemas = schemas.clone();
                html! {
                    <SchemaRegistry items=schemas />
                }
            }
            SubPage::Error(err) => html! { <h1>{{err}}</h1> },
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    yew::initialize();

    let content = document().get_element_by_id("content").unwrap();

    App::<Model>::new().mount(content);

    run_loop();
}

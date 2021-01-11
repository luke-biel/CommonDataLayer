use crate::components::*;
use crate::context_bus::ContextBus;
use yew::prelude::*;
use serde::{Serialize, Deserialize};

pub struct AppContents {
    link: ComponentLink<Self>,
    page: Page,
    _context_bus: Box<dyn Bridge<ContextBus<Page>>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Page {
    Index,
    SchemaRegistry,
}

pub enum Msg {
    RequestPage(Page),
}

impl Component for AppContents {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(Msg::RequestPage);
        let context_bus = ContextBus::<Page>::bridge(callback);

        Self {
            link,
            page: Page::Index,
            _context_bus: context_bus,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::RequestPage(page) => {
                self.page = page;
            }
        }

        true
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        true
    }

    fn view(&self) -> Html {
        html! {
            {
                match self.page {
                    Page::Index => html! { <Index /> },
                    Page::SchemaRegistry => html! { <SchemaRegistry /> },
                }
            }
        }
    }
}

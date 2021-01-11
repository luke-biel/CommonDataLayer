use crate::app_contents::Page;
use crate::context_bus;
use crate::context_bus::ContextBus;
use yew::agent::Dispatcher;
use yew::prelude::*;
use yew_router::prelude::*;

pub struct Menu {
    link: ComponentLink<Self>,
    dispatcher: Dispatcher<ContextBus>,
}

#[derive(Debug, Clone)]
pub enum Msg {
    SchemaRegistry,
}

impl Component for Menu {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            dispatcher: ContextBus::dispatcher(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::SchemaRegistry => self
                .dispatcher
                .send(context_bus::Request::Open(Page::SchemaRegistry)),
        }

        false
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        let open_schema_registry = self.link.callback(|_| Msg::SchemaRegistry);

        html! {
            <>
                { "Mnemosyne" }
                <button onclick=open_schema_registry>
                    { "SCHEMA REGISTRY" }
                </ button>
            </ >
        }
    }
}

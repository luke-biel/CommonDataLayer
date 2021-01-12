use crate::app_contents::Page;
use crate::context_bus;
use crate::context_bus::ContextBus;
use yew::agent::Dispatcher;
use yew::prelude::*;

pub struct Menu {
    link: ComponentLink<Self>,
    dispatcher: Dispatcher<ContextBus<Page>>,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Index,
    SchemaRegistry,
}

impl Component for Menu {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            dispatcher: ContextBus::<Page>::dispatcher(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        let page = match msg {
            Msg::Index => Page::Index,
            Msg::SchemaRegistry => Page::SchemaRegistry,
        };

        self.dispatcher.send(context_bus::Request::Open(page));

        false
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        let open_index = self.link.callback(|_| Msg::Index);
        let open_schema_registry = self.link.callback(|_| Msg::SchemaRegistry);

        html! {
            <>
                { "Mnemosyne" }
                <button onclick=open_index>
                    { "HOME" }
                </ button>
                <button onclick=open_schema_registry>
                    { "SCHEMA REGISTRY" }
                </ button>
            </ >
        }
    }
}

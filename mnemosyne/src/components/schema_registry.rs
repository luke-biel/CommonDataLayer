use uuid::Uuid;
use yew::prelude::*;
use crate::components::*;
use crate::context_bus::ContextBus;

pub struct SchemaRegistry {
    link: ComponentLink<Self>,
    page: Page,
    _context_bus: Box<dyn Bridge<ContextBus<Page>>>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    List,
    View(Uuid),
}

pub enum Msg {
    RequestPage(Page),
}

impl Component for SchemaRegistry {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(Msg::RequestPage);
        let context_bus = ContextBus::<Page>::bridge(callback);

        Self {
            link,
            page: Page::List,
            _context_bus: context_bus,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::RequestPage(page) => {
                if page != self.page {
                    self.page = page;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        match self.page {
            Page::List => html! { <SchemaRegistryList /> },
            Page::View(id) => html! { <SchemaRegistryView id=id /> }
        }
    }
}

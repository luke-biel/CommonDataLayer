use crate::components::*;
use crate::context_bus::ContextBus;
use uuid::Uuid;
use yew::prelude::*;

pub struct SchemaRegistry {
    page: Page, // TODO: Make it a stack
    _context_bus: Box<dyn Bridge<ContextBus<Page>>>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    List,
    View(Uuid),
    Edit(Uuid),
    AddDefinition(Uuid),
    History(Uuid),
    AddSchema,
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
        self.page = Page::List;
        true
    }

    fn view(&self) -> Html {
        match self.page {
            Page::List => html! { <SchemaRegistryList /> },
            Page::View(id) => html! { <SchemaRegistryView id=id /> },
            Page::Edit(id) => html! { <SchemaRegistryEdit id=id /> },
            Page::AddDefinition(id) => html! { <SchemaRegistryAddDefinition id=id /> },
            Page::History(id) => html! { <SchemaRegistryHistory id=id /> },
            Page::AddSchema => html! { <SchemaRegistryAddSchema /> },
        }
    }
}

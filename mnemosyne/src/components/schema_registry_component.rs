use uuid::Uuid;
use yew::prelude::*;

pub struct SchemaRegistryComponent {
    link: ComponentLink<Self>,
    page: Page,
}

pub enum Page {
    List,
    View(Uuid),
}

impl Component for SchemaRegistryComponent {
    type Message = ();
    type Properties = ();

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            page: Page::List,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        html! { {"don goofed"} }
    }
}

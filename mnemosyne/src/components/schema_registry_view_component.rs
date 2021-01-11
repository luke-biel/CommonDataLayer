use uuid::Uuid;
use yew::prelude::*;
use crate::cdl_objects::schema_preview::CDLSchema;

pub struct SchemaRegistryViewComponent {
    link: ComponentLink<Self>,
    state: State,
}

pub enum State {
    Fetching,
    View(CDLSchema),
}

impl Component for SchemaRegistryViewComponent {
    type Message = ();
    type Properties = ();

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            state: State::Fetching,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        html! { }
    }
}

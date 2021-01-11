use uuid::Uuid;
use yew::prelude::*;

pub struct IndexComponent {
    link: ComponentLink<Self>,
}

impl Component for IndexComponent {
    type Message = ();
    type Properties = ();

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        html! { {"xxx"} }
    }
}

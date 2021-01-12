use crate::cdl_objects::schema_preview::CDLSchema;
use crate::GRAPHQL_URL;
use uuid::Uuid;
use yew::prelude::*;
use yewtil::future::LinkFuture;

pub struct SchemaRegistryAddDefinition {
    props: Props,
}

#[derive(Debug, Clone, Properties)]
pub struct Props {
    pub id: Uuid,
}

impl Component for SchemaRegistryAddDefinition {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
            <form>
                <input type="text" placeholder="x.y.z" />
                <textarea />
                <button type="submit">{ "Add schema definition" }</button>
            </form>
            </>
        }
    }
}

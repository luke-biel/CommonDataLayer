use crate::cdl_objects::schema_preview::{CDLSchema, SchemaPreviewQuery};
use crate::GRAPHQL_URL;
use uuid::Uuid;
use yew::prelude::*;
use yewtil::future::LinkFuture;

pub struct SchemaRegistryView {
    props: Props,
    state: State,
}

#[derive(Debug, Clone, Properties)]
pub struct Props {
    pub id: Uuid,
}

pub enum State {
    Fetching,
    View(CDLSchema),
    Error(String),
}

pub enum Msg {
    SuccessfulFetch(CDLSchema),
    Error(String),
}

impl Component for SchemaRegistryView {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let id = props.id;
        link.send_future(async move {
            match SchemaPreviewQuery::fetch(GRAPHQL_URL.clone(), id).await {
                Ok(schemas) => Msg::SuccessfulFetch(schemas),
                Err(error) => Msg::Error(error),
            }
        });

        Self {
            props,
            state: State::Fetching,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::SuccessfulFetch(schema) => self.state = State::View(schema),
            Msg::Error(error) => self.state = State::Error(error),
        }

        true
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        match self.state {
            State::Fetching => html! { <h1>{ "fetching " }{ self.props.id }</h1> },
            State::View(ref schema) => html! {
                <>
                <h1>{ schema.name.as_str() }</h1>
                <h2>{ schema.id }</h2>
                <h2>{ "topic: " }{ schema.topic.as_str() }</h2>
                <h2>{ "query_address: " }{ schema.query_address.as_str() }</h2>
                <h2>{ "type: " }{ schema.repository_type.as_str() }</h2>
                <h3>{ "version: " }{ schema.definition.version.as_str() }</h3>
                <pre>
                    <code>
                        { schema.definition.body.as_str() }
                    </code>
                </pre>
                </>
            },
            State::Error(ref error) => html! { <h1>{ error }</h1> },
        }
    }
}

use crate::cdl_objects::schema_history::{CDLSchemaDefinition, SchemaHistoryQuery};
use crate::GRAPHQL_URL;
use uuid::Uuid;
use yew::prelude::*;
use yewtil::future::LinkFuture;

pub struct SchemaRegistryHistory {
    props: Props,
    state: State,
}

#[derive(Debug, Clone, Properties)]
pub struct Props {
    pub id: Uuid,
}

pub enum State {
    Fetching,
    View(Vec<CDLSchemaDefinition>),
    Error(String),
}

pub enum Msg {
    SuccessfulFetch(Vec<CDLSchemaDefinition>),
    Error(String),
}

impl Component for SchemaRegistryHistory {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let id = props.id;
        link.send_future(async move {
            match SchemaHistoryQuery::fetch(GRAPHQL_URL.clone(), id).await {
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
            State::View(ref schema) => schema
                .iter()
                .map(|def| {
                    html! {
                        <>
                        <h2>{ def.version.as_str() }</h2>
                        <pre>
                            <code>{ def.body.as_str() }</code>
                        </pre>
                        </>
                    }
                })
                .collect::<Html>(),
            State::Error(ref error) => html! { <h1>{ error }</h1> },
        }
    }
}

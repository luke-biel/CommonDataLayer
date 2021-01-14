use crate::cdl_objects::schema_preview::{CDLSchema, SchemaPreviewQuery};
use crate::{cdl_objects, GRAPHQL_URL};
use uuid::Uuid;
use yew::prelude::*;
use yewtil::future::LinkFuture;

pub struct SchemaRegistryView {
    state: State,
}

#[derive(Debug, Clone, Properties)]
pub struct Props {
    pub id: Uuid,
}

pub enum State {
    Fetching,
    View(CDLSchema),
    Error(cdl_objects::Error),
}

pub enum Msg {
    SuccessfulFetch(CDLSchema),
    Error(cdl_objects::Error),
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
            State::Fetching => html! {
                <div class="progress-bar striped animated">
                    <span class="progress-bar-green" style="width: 60%;"></span>
                </div>
            },
            State::View(ref schema) => html! {
                <>
                <div class="form-control-group">
                    <div class="form-control">
                        <legend>{ "schema name" }</legend>
                        <strong>{ schema.name.as_str() }</strong>
                    </div>
                    <div class="form-control">
                        <legend>{ "schema id" }</legend>
                        <strong>{ schema.id }</strong>
                    </div>
                </div>
                <div class="form-control-group">
                    <div class="form-control">
                        <legend>{ "topic" }</legend>
                        <strong>{ schema.topic.as_str() }</strong>
                    </div>
                    <div class="form-control">
                        <legend>{ "query address" }</legend>
                        <strong>{ schema.query_address.as_str() }</strong>
                    </div>
                </div>
                <div class="form-control">
                    <legend>{ "repository type" }</legend>
                    <strong>{ schema.repository_type.as_str() }</strong>
                </div>
                <div class="form-control">
                    <legend>{ "schema version" }</legend>
                    <strong>{ schema.definition.version.as_str() }</strong>
                </div>
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

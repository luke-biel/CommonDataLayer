use crate::cdl_objects::all_schemas::{AllSchemasQuery, CDLSchemaData};
use crate::{cdl_objects, GRAPHQL_URL};
use yew::prelude::*;
use yewtil::future::LinkFuture;

mod row_view;

use crate::components::schema_registry::Page;
use crate::context_bus::{ContextBus, Request};
use row_view::RowView;
use yew::agent::Dispatcher;

pub struct SchemaRegistryList {
    link: ComponentLink<Self>,
    state: State,
    dispatcher: Dispatcher<ContextBus<Page>>,
}

pub enum Msg {
    SuccessfulFetch(CDLSchemaData),
    Error(cdl_objects::Error),
    OpenAdd,
}

pub enum State {
    Fetching,
    List(CDLSchemaData),
    Error(cdl_objects::Error),
}

impl Component for SchemaRegistryList {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        link.send_future(async move {
            match AllSchemasQuery::fetch(GRAPHQL_URL.clone()).await {
                Ok(schemas) => Msg::SuccessfulFetch(schemas),
                Err(error) => Msg::Error(error),
            }
        });

        Self {
            link,
            state: State::Fetching,
            dispatcher: ContextBus::dispatcher(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::SuccessfulFetch(mut data) => {
                data.schemas.sort_by(|s1, s2| s1.id.cmp(&s2.id));
                self.state = State::List(data)
            }
            Msg::Error(error) => self.state = State::Error(error),
            Msg::OpenAdd => self.dispatcher.send(Request::Send(Page::AddSchema)),
        }

        true
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        let on_add = self.link.callback(|_| Msg::OpenAdd);

        match self.state {
            State::Fetching => html! {
                <div class="progress-bar striped animated">
                    <span class="progress-bar-green" style="width: 60%;"></span>
                </div>
            },
            State::List(CDLSchemaData { ref schemas }) => {
                html! {
                    <>
                    <h1 class="h3">
                        { "Schemas" }
                        <button type="button" title="Add schema" class="float-right button-primary button-round" onclick=on_add>
                            <svg width="1.2em" height="1.2em" viewBox="0 0 16 16">
                            <path style="fill: #ffffff"
                                  d="M551,713v6a1,1,0,0,0,2,0v-6h6a1,1,0,0,0,0-2h-6v-6a1,1,0,0,0-2,0v6h-6a1,1,0,0,0,0,2h6Z"
                                  transform="translate(-544 -704)"/>
                            </svg>
                        </button>
                    </h1>
                    <table>
                        <thead>
                        <tr>
                            <th>{ "Name" }</th>
                            <th>{ "Unique Id" }</th>
                            <th>{ "Actions" }</th>
                        </tr>
                        </thead>
                        <tbody>
                        {
                            schemas.iter().map(|schema| {
                                html! { <RowView schema=schema /> }
                            }).collect::<Html>()
                        }
                        </tbody>
                    </table>
                    </>
                }
            }
            State::Error(ref error) => html! { <h1>{ error }</h1> },
        }
    }
}

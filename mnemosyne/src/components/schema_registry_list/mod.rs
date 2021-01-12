use crate::cdl_objects::all_schemas::CDLSchemaData;
use crate::GRAPHQL_URL;
use yew::prelude::*;
use yewtil::future::LinkFuture;

mod row_view;

use row_view::RowView;

pub struct SchemaRegistryList {
    state: State,
}

pub enum Msg {
    SuccessfulFetch(CDLSchemaData),
    Error(String),
}

pub enum State {
    Fetching,
    List(CDLSchemaData),
    Error(String),
}

impl Component for SchemaRegistryList {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        link.send_future(async move {
            match CDLSchemaData::fetch(GRAPHQL_URL.clone()).await {
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
            Msg::SuccessfulFetch(schemas) => self.state = State::List(schemas),
            Msg::Error(error) => self.state = State::Error(error),
        }

        true
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        match self.state {
            State::Fetching => html! { <h1>{ "Fetching schemas" }</h1> },
            State::List(CDLSchemaData { ref schemas }) => {
                html! {
                    <>
                    <h1>{"Schemas"}</h1>
                    <table class="simple-summary">
                        <tr>
                            <td class="simple-summary-heading">{"Name"}</td>
                            <td class="simple-summary-heading">{"Unique Id"}</td>
                            <td class="simple-summary-heading">{"Actions"}</td>
                        </tr>
                        {
                            schemas.iter().map(|schema| {
                                html! { <RowView schema=schema /> }
                            }).collect::<Html>()
                        }
                    </table>
                    </>
                }
            }
            State::Error(ref error) => html! { <h1>{ error }</h1> },
        }
    }
}

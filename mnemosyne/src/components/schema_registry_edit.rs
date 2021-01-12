use crate::cdl_objects::schema_preview::CDLSchema;
use crate::cdl_objects::update_query_address::CDLUpdateQueryAddress;
use crate::cdl_objects::update_topic::CDLUpdateTopic;
use crate::GRAPHQL_URL;
use std::fmt;
use uuid::Uuid;
use yew::prelude::*;
use yewtil::future::LinkFuture;

#[derive(Clone, Debug)]
pub struct SchemaRegistryEdit {
    link: ComponentLink<Self>,
    props: Props,
    state: State,
    topic_form: String,
    query_address_form: String,
}

#[derive(Debug, Clone, Properties)]
pub struct Props {
    pub id: Uuid,
}

#[derive(Clone, Debug)]
enum State {
    Fetching,
    Edit {
        schema: CDLSchema,
        topic: EditState,
        query_address: EditState,
    },
    Error(String),
}

pub enum Msg {
    SuccessfulFetch(CDLSchema),
    Error(String),
    EditTopic,
    EditQueryAddress,
    UpdateTopic(String),
    UpdateQueryAddress(String),
    TopicUpdated(EditState),
    QueryAddressUpdated(EditState),
}

#[derive(Clone, Debug)]
pub enum EditState {
    Void,
    Edited(String),
    Errored(String),
}

impl fmt::Display for EditState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditState::Void => f.write_str(""),
            EditState::Edited(val) => write!(f, "edited {}", val),
            EditState::Errored(err) => write!(f, "failed {}", err),
        }
    }
}

impl Component for SchemaRegistryEdit {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let id = props.id;
        link.send_future(async move {
            match CDLSchema::fetch(GRAPHQL_URL.clone(), id).await {
                Ok(schemas) => Msg::SuccessfulFetch(schemas),
                Err(error) => Msg::Error(error),
            }
        });

        Self {
            link,
            props,
            state: State::Fetching,
            topic_form: "".to_string(),
            query_address_form: "".to_string(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::SuccessfulFetch(schema) => {
                self.state = State::Edit {
                    schema,
                    topic: EditState::Void,
                    query_address: EditState::Void,
                }
            }
            Msg::Error(error) => self.state = State::Error(error),
            Msg::EditTopic => {
                let id = self.props.id;
                let topic = self.topic_form.clone();
                self.link.send_future(async move {
                    match CDLUpdateTopic::fetch(GRAPHQL_URL.clone(), id, topic).await {
                        Ok(topic) => Msg::TopicUpdated(EditState::Edited(topic)),
                        Err(err) => Msg::TopicUpdated(EditState::Errored(err)),
                    }
                })
            }
            Msg::EditQueryAddress => {
                let id = self.props.id;
                let query_address = self.query_address_form.clone();
                self.link.send_future(async move {
                    match CDLUpdateQueryAddress::fetch(GRAPHQL_URL.clone(), id, query_address).await
                    {
                        Ok(query_address) => {
                            Msg::QueryAddressUpdated(EditState::Edited(query_address))
                        }
                        Err(err) => Msg::QueryAddressUpdated(EditState::Errored(err)),
                    }
                })
            }
            Msg::UpdateTopic(val) => self.topic_form = val,
            Msg::UpdateQueryAddress(val) => self.query_address_form = val,
            Msg::TopicUpdated(change) => {
                if let State::Edit { ref mut topic, .. } = self.state {
                    *topic = change;
                } else {
                    log::error!(
                        "Couldn't display topic update. SchemaRegistryEdit was in invalid state"
                    )
                }
            }
            Msg::QueryAddressUpdated(change) => {
                if let State::Edit {
                    ref mut query_address,
                    ..
                } = self.state
                {
                    *query_address = change;
                } else {
                    log::error!(
                        "Couldn't display query_address update. SchemaRegistryEdit was in invalid state"
                    )
                }
            }
        }

        true
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        let on_topic = self.link.callback(|ev: FocusEvent| {
            ev.prevent_default();
            Msg::EditTopic
        });

        let oninput_topic = self
            .link
            .callback(|ev: InputData| Msg::UpdateTopic(ev.value));

        let on_query_address = self.link.callback(|ev: FocusEvent| {
            ev.prevent_default();
            Msg::EditQueryAddress
        });

        let oninput_query_address = self
            .link
            .callback(|ev: InputData| Msg::UpdateQueryAddress(ev.value));

        match self.state {
            State::Fetching => html! { <h1>{ "fetching " }{ self.props.id }</h1> },
            State::Edit {
                ref schema,
                ref topic,
                ref query_address,
            } => html! {
                <>
                <form onsubmit=on_topic>
                    <input type="text" placeholder=schema.topic.as_str() oninput=oninput_topic />
                    <button>{ "CHANGE TOPIC" }</button>
                    <label>{ topic }</label>
                </form>
                <form onsubmit=on_query_address>
                    <input type="text" placeholder=schema.query_address.as_str() oninput=oninput_query_address />
                    <button>{ "CHANGE QUERY ADDRESS" }</button>
                    <label>{ query_address }</label>
                </form>
                </>
            },
            State::Error(ref error) => html! { <h1>{ error }</h1> },
        }
    }
}

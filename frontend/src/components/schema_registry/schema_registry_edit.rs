use crate::cdl_objects::schema_preview::{CDLSchema, SchemaPreviewQuery};
use crate::cdl_objects::update_query_address::UpdateQueryAddressMut;
use crate::cdl_objects::update_topic::UpdateTopicMut;
use crate::components::notification_bar::Notification;
use crate::context_bus::{ContextBus, Request};
use crate::{cdl_objects, GRAPHQL_URL};
use log::Level;
use uuid::Uuid;
use yew::agent::Dispatcher;
use yew::prelude::*;
use yewtil::future::LinkFuture;

pub struct SchemaRegistryEdit {
    link: ComponentLink<Self>,
    props: Props,
    notifications: Dispatcher<ContextBus<Notification>>,
    state: State,
    topic_form: String,
    query_address_form: String,
}

#[derive(Debug, Clone, Properties)]
pub struct Props {
    pub id: Uuid,
}

enum State {
    Fetching,
    Edit(CDLSchema),
    Error(cdl_objects::Error),
}

pub enum Msg {
    SuccessfulFetch(CDLSchema),
    Error(cdl_objects::Error),
    EditTopic,
    EditQueryAddress,
    UpdateTopic(String),
    UpdateQueryAddress(String),
    TopicUpdated(Result<String, cdl_objects::Error>),
    QueryAddressUpdated(Result<String, cdl_objects::Error>),
}

impl Component for SchemaRegistryEdit {
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
            link,
            props,
            notifications: ContextBus::<Notification>::dispatcher(),
            state: State::Fetching,
            topic_form: "".to_string(),
            query_address_form: "".to_string(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::SuccessfulFetch(schema) => self.state = State::Edit(schema),
            Msg::Error(error) => self.state = State::Error(error),
            Msg::EditTopic => {
                let id = self.props.id;
                let topic = self.topic_form.clone();
                self.link.send_future(async move {
                    match UpdateTopicMut::fetch(GRAPHQL_URL.clone(), id, topic).await {
                        Ok(topic) => Msg::TopicUpdated(Ok(topic)),
                        Err(err) => Msg::TopicUpdated(Err(err)),
                    }
                })
            }
            Msg::EditQueryAddress => {
                let id = self.props.id;
                let query_address = self.query_address_form.clone();
                self.link.send_future(async move {
                    match UpdateQueryAddressMut::fetch(GRAPHQL_URL.clone(), id, query_address).await
                    {
                        Ok(query_address) => Msg::QueryAddressUpdated(Ok(query_address)),
                        Err(err) => Msg::QueryAddressUpdated(Err(err)),
                    }
                })
            }
            Msg::UpdateTopic(val) => self.topic_form = val,
            Msg::UpdateQueryAddress(val) => self.query_address_form = val,
            Msg::TopicUpdated(change) => match change {
                Ok(topic) => self.notifications.send(Request::Send(Notification {
                    msg: format!("Topic updated, new topic {}", topic),
                    severity: Level::Info,
                })),
                Err(error) => self.notifications.send(Request::Send(Notification {
                    msg: error.to_string(),
                    severity: Level::Error,
                })),
            },
            Msg::QueryAddressUpdated(change) => match change {
                Ok(qa) => self.notifications.send(Request::Send(Notification {
                    msg: format!("Query address updated, new url {}", qa),
                    severity: Level::Info,
                })),
                Err(error) => self.notifications.send(Request::Send(Notification {
                    msg: error.to_string(),
                    severity: Level::Error,
                })),
            },
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

        match &self.state {
            State::Fetching => html! {
                <div class="progress-bar striped animated">
                    <span class="progress-bar-green" style="width: 60%;"></span>
                </div>
            },
            State::Edit(schema) => html! {
                <>
                <form onsubmit=on_topic>
                    <div class="form-control">
                        <label>{ "Topic" }</label>
                        <input type="text" placeholder=schema.topic.as_str() oninput=oninput_topic />
                    </div>
                    <button class="button-primary button-round">{ "Change topic" }</button>
                </form>
                <form onsubmit=on_query_address>
                    <div class="form-control">
                        <label>{ "Query address" }</label>
                        <input type="text" placeholder=schema.query_address.as_str() oninput=oninput_query_address />
                    </div>
                    <button  class="button-primary button-round">{ "Change query address" }</button>
                </form>
                </>
            },
            State::Error(ref error) => html! { <h1>{ error }</h1> },
        }
    }
}

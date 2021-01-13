use uuid::Uuid;
use yew::prelude::*;
use yewtil::future::LinkFuture;

use crate::cdl_objects::add_definition::AddDefinitionMut;
use crate::components::notification_bar::Notification;
use crate::context_bus::{ContextBus, Request};
use crate::{cdl_objects, GRAPHQL_URL};
use log::Level;
use yew::agent::Dispatcher;

pub struct SchemaRegistryAddDefinition {
    props: Props,
    link: ComponentLink<Self>,
    notifications: Dispatcher<ContextBus<Notification>>,
    form: Form,
}

#[derive(Default)]
pub struct Form {
    version: String,
    definition: String,
}

#[derive(Debug, Clone, Properties)]
pub struct Props {
    pub id: Uuid,
}

pub enum Msg {
    UpdateVersion(String),
    UpdateDefinition(String),
    AddSchema,
    SchemaAdded(Result<String, cdl_objects::Error>),
}

impl Component for SchemaRegistryAddDefinition {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            notifications: ContextBus::<Notification>::dispatcher(),
            form: Default::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateVersion(val) => self.form.version = val,
            Msg::UpdateDefinition(val) => self.form.definition = val,
            Msg::AddSchema => {
                let id = self.props.id;
                let version = self.form.version.clone();
                let definition = self.form.definition.clone();
                self.link.send_future(async move {
                    Msg::SchemaAdded(
                        AddDefinitionMut::fetch(GRAPHQL_URL.clone(), id, version, definition).await,
                    )
                })
            }
            Msg::SchemaAdded(msg) => match msg {
                Ok(def) => self.notifications.send(Request::Send(Notification {
                    msg: format!("Added new schema definition {}", def),
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
        let oninput_version = self
            .link
            .callback(|ev: InputData| Msg::UpdateVersion(ev.value));
        let oninput_definition = self
            .link
            .callback(|ev: InputData| Msg::UpdateDefinition(ev.value));
        let on_submit = self.link.callback(|ev: FocusEvent| {
            ev.prevent_default();
            Msg::AddSchema
        });

        html! {
            <>
            <form onsubmit=on_submit>
                <input type="text" placeholder="x.y.z" oninput=oninput_version/>
                <textarea oninput=oninput_definition />
                <button type="submit">{ "Add schema definition" }</button>
            </form>
            </>
        }
    }
}

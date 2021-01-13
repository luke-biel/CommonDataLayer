use crate::cdl_objects::add_schema::{add_schema_mut::SchemaType, AddSchemaMut};
use crate::components::notification_bar::Notification;
use crate::context_bus::{ContextBus, Request};
use crate::{cdl_objects, GRAPHQL_URL};
use log::Level;
use uuid::Uuid;
use yew::agent::Dispatcher;
use yew::prelude::*;
use yewtil::future::LinkFuture;

pub struct SchemaRegistryAddSchema {
    link: ComponentLink<Self>,
    form: Form,
    notifications: Dispatcher<ContextBus<Notification>>,
}

#[derive(Default)]
struct Form {
    name: String,
    query_address: String,
    topic: String,
    definition: String,
    typ: SchemaType,
}

pub enum Msg {
    UpdateName(String),
    UpdateQueryAddress(String),
    UpdateTopic(String),
    UpdateDefinition(String),
    UpdateTyp(SchemaType),
    AddSchema,
    SchemaAdded(Result<Uuid, cdl_objects::Error>),
}

impl Component for SchemaRegistryAddSchema {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            form: Default::default(),
            notifications: ContextBus::<Notification>::dispatcher(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateName(val) => self.form.name = val,
            Msg::UpdateQueryAddress(val) => self.form.query_address = val,
            Msg::UpdateTopic(val) => self.form.topic = val,
            Msg::UpdateTyp(val) => self.form.typ = val,
            Msg::UpdateDefinition(val) => self.form.definition = val,
            Msg::AddSchema => {
                let name = self.form.name.clone();
                let topic = self.form.topic.clone();
                let query_address = self.form.query_address.clone();
                let typ = self.form.typ.clone();
                let definition = self.form.definition.clone();

                self.link.send_future(async move {
                    match AddSchemaMut::fetch(
                        GRAPHQL_URL.clone(),
                        name,
                        query_address,
                        topic,
                        definition,
                        typ,
                    )
                    .await
                    {
                        Ok(id) => Msg::SchemaAdded(Ok(id)),
                        Err(error) => Msg::SchemaAdded(Err(error)),
                    }
                })
            }
            Msg::SchemaAdded(msg) => match msg {
                Ok(id) => self.notifications.send(Request::Send(Notification {
                    msg: format!("Saved schema under id {}", id),
                    severity: Level::Info,
                })),
                Err(err) => self.notifications.send(Request::Send(Notification {
                    msg: err.to_string(),
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
        let oninput_name = self
            .link
            .callback(|ev: InputData| Msg::UpdateName(ev.value));
        let oninput_topic = self
            .link
            .callback(|ev: InputData| Msg::UpdateTopic(ev.value));
        let oninput_query_address = self
            .link
            .callback(|ev: InputData| Msg::UpdateQueryAddress(ev.value));
        let onchange_typ = self.link.callback(|ev: ChangeData| {
            match ev {
                ChangeData::Value(_) => unimplemented!(), // Can't ever happen
                ChangeData::Select(sel) => Msg::UpdateTyp(sel.value().parse().unwrap()), // Given we use `select` this cannot fail
                ChangeData::Files(_) => unimplemented!(), // Can't ever happen
            }
        });
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
                <div class="form-control">
                    <label>{ "New schema name" }</label>
                    <input type="text" placeholder="schema name" oninput=oninput_name />
                </div>
                <div class="form-control-group">
                    <div class="form-control">
                        <label>{ "Ingestion topic" }</label>
                        <input type="text" placeholder="topic" oninput=oninput_topic />
                    </div>
                    <div class="form-control">
                        <label>{ "Query service address" }</label>
                        <input type="text" placeholder="query address" oninput=oninput_query_address />
                    </div>
                </div>
                <div class="form-control">
                    <label>{ "Repository type" }</label>
                    <select name="storage_type" onchange=onchange_typ >
                        <option hidden=true disabled=true selected=true></option>
                        <option value=SchemaType::DOCUMENT_STORAGE >{ SchemaType::DOCUMENT_STORAGE }</option>
                        <option value=SchemaType::TIMESERIES >{ SchemaType::TIMESERIES }</option>
                    </select>
                </div>
                <div class="form-control">
                    <label>{ "Json Schema" }</label>
                    <textarea oninput=oninput_definition />
                </div>
                <div class="form-control">
                    <button class="button-primary button-round" type="submit">{ "Add schema" }</button>
                </div>
            </form>
            </>
        }
    }
}

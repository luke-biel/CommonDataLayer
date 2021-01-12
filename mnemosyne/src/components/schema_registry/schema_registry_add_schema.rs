use crate::cdl_objects::add_schema::AddSchemaMut;
use crate::GRAPHQL_URL;
use yew::prelude::*;
use yewtil::future::LinkFuture;

pub struct SchemaRegistryAddSchema {
    link: ComponentLink<Self>,
    form: Form,
    msg: String,
}

#[derive(Default)]
struct Form {
    name: String,
    query_address: String,
    topic: String,
    definition: String,
    typ: String,
}

pub enum Msg {
    UpdateName(String),
    UpdateQueryAddress(String),
    UpdateTopic(String),
    UpdateDefinition(String),
    UpdateTyp(String),
    AddSchema,
    SchemaAdded(String),
}

impl Component for SchemaRegistryAddSchema {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            form: Default::default(),
            msg: "".to_string(),
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
                        Ok(id) => Msg::SchemaAdded(id.to_string()),
                        Err(error) => Msg::SchemaAdded(error), // TODO: Change to custom error
                    }
                })
            }
            Msg::SchemaAdded(msg) => self.msg = msg, // TODO: Hold state elsewhere
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
                ChangeData::Select(sel) => Msg::UpdateTyp(sel.value()),
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
                <input type="text" placeholder="schema name" oninput=oninput_name />
                <input type="text" placeholder="schema topic" oninput=oninput_topic />
                <input type="text" placeholder="schema query_address" oninput=oninput_query_address />
                <select name="storage_type" onchange=onchange_typ >
                    <option hidden=true disabled=true selected=true></option>
                    <option value="DOCUMENT_STORAGE">{ "DOCUMENT_STORAGE" }</option>
                    <option value="TIMESERIES">{ "TIMESERIES" }</option>
                </select>
                <textarea oninput=oninput_definition />
                <button type="submit">{ "Add schema" }</button>
                <label>{ self.msg.as_str() }</label>
            </form>
            </>
        }
    }
}

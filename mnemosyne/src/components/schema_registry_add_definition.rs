use crate::cdl_objects::add_definition::{AddDefinitionMut, CDLAddDefinition};
use crate::GRAPHQL_URL;
use uuid::Uuid;
use yew::prelude::*;
use yewtil::future::LinkFuture;

pub struct SchemaRegistryAddDefinition {
    props: Props,
    link: ComponentLink<Self>,
    version: String,
    definition: String,
    msg: String,
}

#[derive(Debug, Clone, Properties)]
pub struct Props {
    pub id: Uuid,
}

pub enum Msg {
    UpdateVersion(String),
    UpdateDefinition(String),
    AddSchema,
    SchemaAdded(String),
}

impl Component for SchemaRegistryAddDefinition {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            version: "".to_string(),
            definition: "".to_string(),
            msg: "".to_string(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateVersion(val) => self.version = val,
            Msg::UpdateDefinition(val) => self.definition = val,
            Msg::AddSchema => {
                let id = self.props.id;
                let version = self.version.clone();
                let definition = self.definition.clone();
                self.link.send_future(async move {
                    match AddDefinitionMut::fetch(GRAPHQL_URL.clone(), id, version, definition)
                        .await
                    {
                        Ok(definition) => Msg::SchemaAdded(definition),
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
                <label>{ self.msg.as_str() }</label>
            </form>
            </>
        }
    }
}

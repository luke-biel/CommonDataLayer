use std::rc::Rc;

use uuid::Uuid;
use yew::html;
use yew::{Component, ComponentLink, Html, Properties};
use yewtil::future::LinkFuture;

use crate::cdl_objects::all_schemas::{CDLSchemaView, CDLSchemas};
use crate::cdl_objects::schema_preview::CDLSchema;
use crate::{FetchState, Model, Msg};
use reqwest::Url;
use std::future::Future;
use std::sync::Arc;

#[derive(Clone, Debug, Properties)]
pub struct SchemaRegistryProps {
    pub items: Rc<CDLSchemas>,
    pub model: ComponentLink<Model>,
}

pub struct SchemaRegistry {
    props: SchemaRegistryProps,
}

impl Component for SchemaRegistry {
    type Message = ();
    type Properties = SchemaRegistryProps;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _msg: Self::Message) -> bool {
        false
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        if !Rc::ptr_eq(&self.props.items, &props.items) {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let model = &self.props.model;

        html! {
            <>
            <h1>{"Schemas"}</h1>
            <table class="simple-summary">
                <tr>
                    <td class="simple-summary-heading">{"Name"}</td>
                    <td class="simple-summary-heading">{"Unique Id"}</td>
                    <td class="simple-summary-heading">{"Actions"}</td>
                </tr>
                { self.props.items.schemas.iter().map(|schema| {
                    html! { <RowView item=schema model=model.clone() /> }
                } ).collect::<Html>() }
            </table>
            </>
        }
    }
}

struct RowView {
    props: RowViewProps,
    link: ComponentLink<Self>,
}

#[derive(Clone, Debug, Properties)]
struct RowViewProps {
    item: Arc<CDLSchemaView>,
    model: ComponentLink<Model>,
}

enum RowViewMsg {
    OpenView,
    OpenEdit,
}

impl Component for RowView {
    type Message = RowViewMsg;
    type Properties = RowViewProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { props, link }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        let item = Arc::clone(&self.props.item);

        match msg {
            RowViewMsg::OpenView => self.props.model.send_future(async move {
                match CDLSchema::fetch(
                    Url::parse("http://localhost:50106/graphql").unwrap(),
                    item.id,
                )
                .await
                {
                    Ok(schema) => Msg::SetFetchState(FetchState::Schema(schema)),
                    Err(err) => Msg::SetFetchState(FetchState::Failed(err)),
                }
            }),
            RowViewMsg::OpenEdit => {
                todo!()
            }
        }

        false
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        if !Arc::ptr_eq(&self.props.item, &props.item) {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let open_view = self.link.callback(|_| RowViewMsg::OpenView);

        let item = self.props.item.clone();

        html! {
            <tr>
                <td class="simple-name-column simple-column">{ item.name.as_str() }</td>
                <td class="simple-column">{ item.id }</td>
                <td class="simple-column">
                    <button type="button" title="View schema" class="small-action-button" onclick=open_view>
                        <svg width="1.2em" height="1.2em" viewBox="0 0 10 10">
                            <path class="small-svg-button"
                                d="M588.6,747.293l-3.115-3.141A3.455,3.455,0,1,0,582.438,746a3.4,3.4,0,0,0,1.639-.434l3.116,3.141a0.985,0.985,0,0,0,1.4,0A1,1,0,0,0,588.6,747.293Zm-7.644-4.793a1.488,1.488,0,1,1,1.487,1.5A1.494,1.494,0,0,1,580.951,742.5Z"
                                transform="translate(-578.969 -739)"/>
                        </svg>
                    </button>
                    <button type="button" title="Edit schema" class="small-action-button">
                        <svg width="1.2em" height="1.2em" viewBox="0 0 16 16">
                        <path class="small-svg-button"
                              d="M78.993,432H65.007a1,1,0,0,1-1-1V417a1,1,0,0,1,1-1H71a1,1,0,0,1,0,2h-5v12H77.994v-5a1,1,0,1,1,2,0v6A1,1,0,0,1,78.993,432Zm-2-14,1,1L71,426H70v-1l6.993-7h0Zm0-2a1.989,1.989,0,0,0-1.413.586l-6.993,7A2,2,0,0,0,68,425v1a2,2,0,0,0,2,2h1a2,2,0,0,0,1.413-.586l6.993-7a2,2,0,0,0,0-2.828l-1-1A1.989,1.989,0,0,0,76.995,416h0Z"
                              transform="translate(-64 -416)"/>
                        </svg>
                    </button>
                </td>
            </tr>
        }
    }
}

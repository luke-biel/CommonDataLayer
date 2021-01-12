use crate::cdl_objects::all_schemas::CDLSchemaView;
use crate::components::schema_registry::Page;
use crate::context_bus::{ContextBus, Request};
use yew::agent::Dispatcher;
use yew::prelude::*;

pub struct RowView {
    props: Props,
    link: ComponentLink<Self>,
    dispatcher: Dispatcher<ContextBus<Page>>,
}

#[derive(Clone, Debug, Properties)]
pub struct Props {
    pub schema: CDLSchemaView,
}

pub enum Msg {
    OpenView,
    OpenEdit,
    OpenAdd,
    OpenHistory,
}

impl Component for RowView {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            dispatcher: ContextBus::<Page>::dispatcher(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        let page = match msg {
            Msg::OpenView => Page::View(self.props.schema.id),
            Msg::OpenEdit => Page::Edit(self.props.schema.id),
            Msg::OpenAdd => Page::AddDefinition(self.props.schema.id),
            Msg::OpenHistory => Page::History(self.props.schema.id),
        };

        self.dispatcher.send(Request::Open(page));

        false
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        let on_view = self.link.callback(|_| Msg::OpenView);
        let on_edit = self.link.callback(|_| Msg::OpenEdit);
        let on_add = self.link.callback(|_| Msg::OpenAdd);
        let on_history = self.link.callback(|_| Msg::OpenHistory);

        html! {
            <tr>
                <td class="simple-name-column simple-column">{ self.props.schema.name.as_str() }</td>
                <td class="simple-column">{ self.props.schema.id }</td>
                <td class="simple-column">
                    <button type="button" title="View schema" class="small-action-button" onclick=on_view>
                        <svg width="1.2em" height="1.2em" viewBox="0 0 10 10">
                            <path class="small-svg-button"
                                d="M588.6,747.293l-3.115-3.141A3.455,3.455,0,1,0,582.438,746a3.4,3.4,0,0,0,1.639-.434l3.116,3.141a0.985,0.985,0,0,0,1.4,0A1,1,0,0,0,588.6,747.293Zm-7.644-4.793a1.488,1.488,0,1,1,1.487,1.5A1.494,1.494,0,0,1,580.951,742.5Z"
                                transform="translate(-578.969 -739)"/>
                        </svg>
                    </button>
                    <button type="button" title="Edit schema" class="small-action-button" onclick=on_edit>
                        <svg width="1.2em" height="1.2em" viewBox="0 0 16 16">
                        <path class="small-svg-button"
                              d="M78.993,432H65.007a1,1,0,0,1-1-1V417a1,1,0,0,1,1-1H71a1,1,0,0,1,0,2h-5v12H77.994v-5a1,1,0,1,1,2,0v6A1,1,0,0,1,78.993,432Zm-2-14,1,1L71,426H70v-1l6.993-7h0Zm0-2a1.989,1.989,0,0,0-1.413.586l-6.993,7A2,2,0,0,0,68,425v1a2,2,0,0,0,2,2h1a2,2,0,0,0,1.413-.586l6.993-7a2,2,0,0,0,0-2.828l-1-1A1.989,1.989,0,0,0,76.995,416h0Z"
                              transform="translate(-64 -416)"/>
                        </svg>
                    </button>
                    <button type="button" title="Add schema" class="small-action-button" onclick=on_add>
                        <svg width="1.2em" height="1.2em" viewBox="0 0 16 16">
                        <path class="small-svg-button"
                              d="M551,713v6a1,1,0,0,0,2,0v-6h6a1,1,0,0,0,0-2h-6v-6a1,1,0,0,0-2,0v6h-6a1,1,0,0,0,0,2h6Z"
                              transform="translate(-544 -704)"/>
                        </svg>
                    </button>
                    <button type="button" title="View schema history" class="small-action-button" onclick=on_history>
                        <svg width="1.2em" height="1.2em" viewBox="0 0 16 16">
                        <path class="small-svg-button"
                              d="M557,416h-9a2,2,0,0,0-2,2v1a1,1,0,0,0,0,2v2a1,1,0,0,0,0,2v2a1,1,0,0,0,0,2v1a2,2,0,0,0,2,2h9a2,2,0,0,0,2-2V418A2,2,0,0,0,557,416Zm0,14h-9v-1a1,1,0,0,0,0-2v-2a1,1,0,0,0,0-2v-2a1,1,0,0,0,0-2v-1h9v12Zm-6-8h3a1,1,0,0,0,0-2h-3A1,1,0,0,0,551,422Z"
                              transform="translate(-545 -416)"/>
                        </svg>
                    </button>
                </td>
            </tr>
        }
    }
}

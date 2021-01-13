use crate::context_bus::ContextBus;
use log::Level;
use yew::prelude::*;

pub struct NotificationBar {
    stack: Vec<Notification>,
    _context_bus: Box<dyn Bridge<ContextBus<Notification>>>,
}

#[derive(Clone)]
pub struct Notification {
    pub(crate) msg: String,
    pub(crate) severity: Level,
}

pub enum Msg {
    Push(Notification),
}

impl Component for NotificationBar {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(Msg::Push);

        Self {
            stack: Vec::new(),
            _context_bus: ContextBus::<Notification>::bridge(callback),
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::Push(notification) => {
                if self.stack.len() >= 5 {
                    self.stack.pop();
                }
                self.stack.insert(0, notification);

                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        self.stack
            .iter()
            .map(|n| {
                html! {
                    <div class="card">
                        <div class="card-title">
                            { n.severity }
                        </div>
                        <p>
                            { n.msg.as_str() }
                        </p>
                    </div>
                }
            })
            .collect::<Html>()
    }
}

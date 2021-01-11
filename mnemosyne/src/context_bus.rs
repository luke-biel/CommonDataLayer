use crate::app_contents::Page;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use yew::worker::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Open(Page),
}

pub struct ContextBus {
    link: AgentLink<ContextBus>,
    subscribers: HashSet<HandlerId>,
}

impl Agent for ContextBus {
    type Reach = Context<Self>;
    type Message = ();
    type Input = Request;
    type Output = Page;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            Request::Open(page) => {
                for sub in self.subscribers.iter().copied() {
                    self.link.respond(sub, page);
                }
            }
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}

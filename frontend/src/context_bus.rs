use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::marker::PhantomData;
use yew::worker::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request<T: Clone + 'static> {
    Send(T),
}

pub struct ContextBus<T: Clone + 'static> {
    link: AgentLink<ContextBus<T>>,
    subscribers: HashSet<HandlerId>,
    _phantom: PhantomData<T>,
}

impl<T: Clone + 'static> Agent for ContextBus<T> {
    type Reach = Context<Self>;
    type Message = ();
    type Input = Request<T>;
    type Output = T;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
            _phantom: PhantomData,
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            Request::Send(page) => {
                for sub in self.subscribers.iter().copied() {
                    self.link.respond(sub, page.clone());
                }
            }
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}

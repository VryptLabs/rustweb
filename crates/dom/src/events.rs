use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct DelegatedEvent {
    pub event_type: String,

    pub handler_id: String,

    pub target_hid: Option<String>,

    pub payload_json: String,
}

impl DelegatedEvent {
    pub fn click(handler_id: impl Into<String>) -> Self {
        Self {
            event_type: "click".into(),
            handler_id: handler_id.into(),
            target_hid: None,
            payload_json: "{}".into(),
        }
    }
}

pub struct EventDelegator<H = Rc<dyn Fn(DelegatedEvent)>> {
    handlers: HashMap<(String, String), H>,
}

impl<H> Default for EventDelegator<H> {
    fn default() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
}

impl<H> EventDelegator<H> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        event: impl Into<String>,
        handler_id: impl Into<String>,
        handler: H,
    ) -> Option<H> {
        self.handlers
            .insert((event.into(), handler_id.into()), handler)
    }

    pub fn unregister(&mut self, event: &str, handler_id: &str) -> bool {
        self.handlers
            .remove(&(event.to_string(), handler_id.to_string()))
            .is_some()
    }

    pub fn binding_count(&self) -> usize {
        self.handlers.len()
    }

    pub fn root_listener_count(&self) -> usize {
        let mut kinds = std::collections::HashSet::new();
        for (ev, _) in self.handlers.keys() {
            kinds.insert(ev.clone());
        }
        kinds.len()
    }

    pub fn get(&self, event: &str, handler_id: &str) -> Option<&H> {
        self.handlers
            .get(&(event.to_string(), handler_id.to_string()))
    }
}

impl EventDelegator<Rc<dyn Fn(DelegatedEvent)>> {
    pub fn dispatch(&self, ev: &DelegatedEvent) -> bool {
        match self.get(&ev.event_type, &ev.handler_id) {
            Some(h) => {
                let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (h)(ev.clone())));
                match r {
                    Ok(()) => true,
                    Err(_) => {
                        eprintln!(
                            "[rustweb] event handler `{}` panicked; isolated",
                            ev.handler_id
                        );
                        false
                    }
                }
            }
            None => false,
        }
    }
}

#[cfg(all(target_arch = "wasm32", feature = "default"))]
mod wasm_attach {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delegation_counts_prove_single_root_listener() {
        let mut d: EventDelegator<String> = EventDelegator::new();
        for i in 0..100 {
            d.register("click", format!("h{i}"), format!("handler{i}"));
        }
        assert_eq!(d.binding_count(), 100);
        assert_eq!(
            d.root_listener_count(),
            1,
            "100 buttons must still need 1 root listener"
        );
    }

    #[test]
    fn dispatch_isolates_panic() {
        let mut d: EventDelegator<Rc<dyn Fn(DelegatedEvent)>> = EventDelegator::new();
        d.register(
            "click",
            "boom",
            Rc::new(|_: DelegatedEvent| -> () { panic!("handler bug") })
                as Rc<dyn Fn(DelegatedEvent)>,
        );
        let ok = d.dispatch(&DelegatedEvent::click("boom"));
        assert!(!ok, "panicking handler must be isolated, not propagate");
    }
}

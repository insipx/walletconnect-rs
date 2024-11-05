//! Event system based on tokio broadcast mpsc and tokio::spawn

use std::collections::HashMap;

use tokio::sync::mpsc;

enum Ev {
    TestEv,
    RegisterEvent { event: Box<Self>, fun: Box<dyn Fn()> },
}

pub struct Events {
    sender: mpsc::Sender<Ev>,
    handle: tokio::task::AbortHandle,
}

impl Events {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(10);

        let inner = EventsInner::new(rx);
        let handle = tokio::task::spawn(async move {
            loop {
                let ev = inner.rx.recv().await;
                if let Some(Ev::RegisterEvent { event, fun }) = ev {
                    inner.push_event(event, fun)
                }
            }
        });

        Self { sender: tx, handle: handle.abort_handle() }
    }

    /// Get a sender handle to these events
    pub fn sender(&self) -> mpsc::Sender<Ev> {
        self.sender.clone()
    }
}

impl Drop for Events {
    fn drop(&mut self) {
        self.handle.abort()
    }
}

struct EventsInner {
    rx: mpsc::Receiver<Ev>,
    events: HashMap<Ev, Vec<Box<dyn Fn()>>>,
}

impl EventsInner {
    pub fn new(rx: mpsc::Receiver<Ev>) -> Self {
        Self { rx, events: HashMap::new() }
    }

    fn push_event(&mut self, event: Ev, on_event: impl Fn()) {
        match self.events.get_mut(event) {
            Some(ref mut events) => events.push(Box::new(on_event)),
            None => self.events.insert(event, vec![on_event]),
        }
    }

    fn run(&self, event: &Ev) {
        self.get(event)()
    }
}

use crate::GlobalEvent;

pub struct GlobalEvents {
    sender: tokio::sync::broadcast::Sender<crate::GlobalEvent>,
}

impl GlobalEvents {
    pub fn emit<E: Into<super::GlobalEvent>>(&self, event: E) {
        let _ = self.sender.send(event.into());
    }

    pub fn register<F: Fn(&GlobalEvent) + Send + 'static>(&self, f: F) {
        let mut receiver = self.sender.subscribe();
        tokio::spawn(async move {
            loop {
                //TODO: Handle lagging
                let ev = receiver.recv().await.unwrap();
                f(&ev)
            }
        });
    }
}

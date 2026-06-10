use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Single-frame pub/sub channels cleared after each tick.
pub struct Events {
    channels: HashMap<TypeId, Vec<Box<dyn Any + Send>>>,
}

impl Default for Events {
    fn default() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }
}

impl Events {
    pub fn send<T: Send + 'static>(&mut self, event: T) {
        self.channels
            .entry(TypeId::of::<T>())
            .or_default()
            .push(Box::new(event));
    }

    pub fn drain<T: Send + 'static>(&mut self) -> Vec<T> {
        self.channels
            .remove(&TypeId::of::<T>())
            .map(|boxed| {
                boxed
                    .into_iter()
                    .map(|event| *event.downcast::<T>().expect("event type mismatch"))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn clear(&mut self) {
        self.channels.clear();
    }
}

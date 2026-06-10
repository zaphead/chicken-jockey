use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Type-erased resource storage keyed by `TypeId`.
pub struct Resources {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl Resources {
    pub fn insert<T: Send + Sync + 'static>(&mut self, resource: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(resource));
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref())
    }

    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|value| value.downcast_mut())
    }

    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.map
            .remove(&TypeId::of::<T>())
            .and_then(|value| value.downcast().ok())
            .map(|boxed| *boxed)
    }
}

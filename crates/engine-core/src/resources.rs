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

    /// Borrows two distinct resources when one is mutably accessed.
    pub fn with_pair<I: Send + Sync + 'static, M: Send + Sync + 'static, R>(
        &mut self,
        f: impl FnOnce(&I, &mut M) -> R,
    ) -> Option<R> {
        let i_id = TypeId::of::<I>();
        let m_id = TypeId::of::<M>();
        if i_id == m_id {
            return None;
        }

        let i_ptr = self.map.get(&i_id).map(|value| value.as_ref() as *const dyn Any)?;
        let m_ptr = self
            .map
            .get_mut(&m_id)
            .map(|value| value.as_mut() as *mut dyn Any)?;

        unsafe {
            let immutable = &*(*i_ptr).downcast_ref::<I>().expect("resource type");
            let mutable = &mut *(*m_ptr).downcast_mut::<M>().expect("resource type");
            Some(f(immutable, mutable))
        }
    }

    /// Borrows three distinct resources (two immutable, one mutable).
    pub fn with_triple<A: Send + Sync + 'static, B: Send + Sync + 'static, C: Send + Sync + 'static, R>(
        &mut self,
        f: impl FnOnce(&A, &B, &mut C) -> R,
    ) -> Option<R> {
        let a_id = TypeId::of::<A>();
        let b_id = TypeId::of::<B>();
        let c_id = TypeId::of::<C>();
        if a_id == b_id || a_id == c_id || b_id == c_id {
            return None;
        }

        let a_ptr = self.map.get(&a_id).map(|value| value.as_ref() as *const dyn Any)?;
        let b_ptr = self.map.get(&b_id).map(|value| value.as_ref() as *const dyn Any)?;
        let c_ptr = self
            .map
            .get_mut(&c_id)
            .map(|value| value.as_mut() as *mut dyn Any)?;

        unsafe {
            let a = &*(*a_ptr).downcast_ref::<A>().expect("resource type");
            let b = &*(*b_ptr).downcast_ref::<B>().expect("resource type");
            let c = &mut *(*c_ptr).downcast_mut::<C>().expect("resource type");
            Some(f(a, b, c))
        }
    }
}

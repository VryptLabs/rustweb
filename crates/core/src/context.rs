use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct ContextMap {
    inner: HashMap<TypeId, Rc<dyn Any>>,
}

impl std::fmt::Debug for ContextMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextMap").field("entries", &self.inner.len()).finish()
    }
}

impl ContextMap {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<T: Clone + 'static>(&mut self, value: T) {
        self.inner.insert(TypeId::of::<T>(), Rc::new(value));
    }

    pub fn get<T: Clone + 'static>(&self) -> Option<T> {
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|rc| rc.downcast_ref::<T>())
            .cloned()
    }

    pub fn contains<T: 'static>(&self) -> bool {
        self.inner.contains_key(&TypeId::of::<T>())
    }
}

#[derive(Debug, Clone)]
pub struct ContextProvider<T: Clone + 'static> {

    pub value: T,
}

impl<T: Clone + 'static> ContextProvider<T> {

    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn provide(&self, mut ambient: ContextMap) -> ContextMap {
        ambient.insert(self.value.clone());
        ambient
    }
}

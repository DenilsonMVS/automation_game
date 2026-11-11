use std::sync::Arc;

use arc_swap::ArcSwap;


#[derive(Debug)]
pub struct Producer<T> {
    shared: Arc<ArcSwap<T>>,
}

#[derive(Debug, Clone)]
pub struct Consumer<T> {
    shared: Arc<ArcSwap<T>>,
}

pub fn channel<T>(initial_state: T) -> (Producer<T>, Consumer<T>) {
    let shared = Arc::new(ArcSwap::from_pointee(initial_state));
    (
        Producer { shared: shared.clone() },
        Consumer { shared },
    )
}

impl<T> Producer<T> {
    pub fn publish(&mut self, snapshot: T) {
        self.shared.store(Arc::new(snapshot));
    }
}

impl<T> Consumer<T> {
    pub fn acquire(&self) -> Arc<T> {
        self.shared.load_full()
    }
}

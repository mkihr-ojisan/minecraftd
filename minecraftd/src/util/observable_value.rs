pub struct ObservableValue<T: Clone + 'static> {
    tx: tokio::sync::watch::Sender<T>,
}

impl<T: Clone + 'static> ObservableValue<T> {
    pub fn new(initial_value: T) -> Self {
        let (tx, _) = tokio::sync::watch::channel(initial_value);
        Self { tx }
    }

    pub fn get(&self) -> T {
        self.tx.borrow().clone()
    }

    pub fn set(&self, value: T) {
        let _ = self.tx.send_replace(value);
    }

    pub fn wait_until<F>(&self, predicate: F) -> impl Future<Output = anyhow::Result<()>> + 'static
    where
        F: Fn(&T) -> bool + 'static,
    {
        let mut rx = self.tx.subscribe();
        async move {
            loop {
                if predicate(&rx.borrow_and_update()) {
                    return Ok(());
                }
                rx.changed().await?;
            }
        }
    }
}

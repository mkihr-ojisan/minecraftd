use std::ops::Deref;

pub struct LazyInitHttpClient {
    client: std::sync::LazyLock<reqwest::Client>,
}

impl LazyInitHttpClient {
    pub const fn new() -> Self {
        Self {
            client: std::sync::LazyLock::new(reqwest::Client::new),
        }
    }
}

impl Deref for LazyInitHttpClient {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

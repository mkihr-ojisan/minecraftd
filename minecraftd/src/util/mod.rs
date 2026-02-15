use std::pin::Pin;

pub mod lazy_init_http_client;
pub mod observable_value;

pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

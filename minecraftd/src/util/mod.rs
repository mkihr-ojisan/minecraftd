use std::pin::Pin;

pub mod cached_mojang_piston_api;
pub mod lazy_init_http_client;
pub mod observable_value;
pub mod os_str_ext;

pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

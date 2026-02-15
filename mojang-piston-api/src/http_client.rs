use std::sync::OnceLock;

use reqwest::Client;

static CLIENT: OnceLock<Client> = OnceLock::new();

pub fn http_client() -> &'static Client {
    CLIENT.get_or_init(Client::new)
}

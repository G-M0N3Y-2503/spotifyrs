#![warn(missing_docs)]
//! Rust bindings for authorising and using the [Spotify Web API](https://developer.spotify.com/documentation/web-api/reference/#/)

use lazy_static::lazy_static;
use reqwest::*;

lazy_static! {
    static ref CLIENT: reqwest::Client = ClientBuilder::new()
        .build()
        .expect("WASM client should succeed");
    static ref ENDPOINT: utils::Url = Url::parse("https://api.spotify.com/v1")
        .expect("A valid API endpoint")
        .try_into()
        .expect("A base URL");
}

pub mod authorisation;
mod client;
pub use client::Client;

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_endpoints() {
        assert_eq!(ENDPOINT.as_str(), "https://api.spotify.com/v1");
        assert_eq!(
            ENDPOINT.with_path([""]).as_str(),
            "https://api.spotify.com/v1/"
        );
        assert_eq!(
            ENDPOINT.with_path(["path"]).as_str(),
            "https://api.spotify.com/v1/path"
        );
    }
}

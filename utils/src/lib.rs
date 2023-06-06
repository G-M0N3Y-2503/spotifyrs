#![warn(missing_docs)]
//! [`web_sys`] helper operations

mod store;
use base64::Engine;
pub use store::*;
mod url;
pub use crate::url::*;
mod delay;
pub mod request;
pub use delay::*;
pub use request::request;

/// Returns a URL Safe base64 encoded `String` of the given `bytes`
pub fn base64(bytes: &[u8]) -> String {
    use base64::{
        alphabet::URL_SAFE,
        engine::general_purpose::{GeneralPurpose, NO_PAD},
    };
    GeneralPurpose::new(&URL_SAFE, NO_PAD).encode(bytes)
}

/// Returns the given amount of cryptographically random bytes as aa base64 encoded string
/// The amount of chars returned will be `ceil(bytes * 8 / 6)`
pub fn random(bytes: u16) -> String {
    let mut random = Vec::with_capacity(bytes.into());
    random.resize(bytes.into(), Default::default());
    web_sys::window()
        .unwrap()
        .crypto()
        .unwrap()
        .get_random_values_with_u8_array(random.as_mut_slice())
        .expect("bytes is less than or equal to 65,536");

    base64(&random)
}

/// Hashes the given data with SHA-256
pub async fn sha256(data: &[u8]) -> String {
    base64(
        &js_sys::Uint8Array::new(
            &wasm_bindgen_futures::JsFuture::from(
                web_sys::window()
                    .unwrap()
                    .crypto()
                    .unwrap()
                    .subtle()
                    .digest_with_str_and_u8_array("SHA-256", data.to_owned().as_mut_slice())
                    .unwrap(),
            )
            .await
            .unwrap(),
        )
        .to_vec(),
    )
}

/// Browser window wrapper helper consistant errors when using in an invalid context
///
/// # Panics
/// If the browser window doesn't exist.
/// This is typically when used in an invalid context.
pub fn browser_window() -> web_sys::Window {
    web_sys::window().expect("Browser window doesn't exist")
}

/// Returns the current browser location
///
/// # Panics
/// If the browser window doesn't exist or the URL isn't syntactically correct.
/// This is typically when used in an invalid context.
pub fn browser_location() -> ::url::Url {
    ::url::Url::parse(
        &browser_window()
            .location()
            .href()
            .expect("Browser is at a valid URL"),
    )
    .expect("A valid browser location")
}

/// Helper struct for displaying mismatched types
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Mismatch<T: std::fmt::Display> {
    /// The expected value
    pub expected: T,
    /// The received value
    pub received: T,
}
impl<T: std::fmt::Display> std::fmt::Display for Mismatch<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "expected \"{}\", but received \"{}\"",
            self.expected, self.received
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_sha256() {
        const TEST_RUNS: usize = 10;
        for _ in 1..=TEST_RUNS {
            assert_eq!(
                &sha256("".as_bytes()).await,
                "47DEQpj8HBSa-_TImW-5JCeuQeRkm5NMpJWZG3hSuFU"
            );
            assert_eq!(
                &sha256("abc123".as_bytes()).await,
                "bKE9UspwyIPg8LsQHkJaiehiTeUdstI5JZOvaoQRgJA"
            );
            assert_eq!(
                &sha256("12345678910121314151617181920212324252627282930".as_bytes()).await,
                "iEyGxp2ANv3HElRj2cn8yp8Nd3JY68AhKEyJGMPdXjk"
            );
        }
    }

    #[wasm_bindgen_test]
    fn test_random() {
        const TEST_RUNS: usize = 10_000;
        let mut set = std::collections::HashSet::<String>::with_capacity(TEST_RUNS);
        for _ in 1..=TEST_RUNS {
            let random = random(32);
            assert!(
                set.insert(random.clone()),
                "non-unique random value generated: \"{random}\""
            );
        }
        for bytes in 1..1024 {
            assert_eq!(
                random(bytes).len(),
                ((bytes * 8) as f64 / 6f64).ceil() as usize
            );
        }
    }
}

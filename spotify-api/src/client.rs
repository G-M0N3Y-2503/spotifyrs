use super::authorisation::*;
use instant::Duration;
use utils::request::*;

/// A client that abstracts the need for refreshing the [AccessToken] and authorises each API [request]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Client {
    token: AccessToken,
}

impl Client {
    /// Creates a client consuming the [AccessToken]
    pub fn new(token: AccessToken) -> Self {
        Self { token }
    }

    /// Disposes of the Client and returns the [AccessToken]
    pub fn take_token(self) -> AccessToken {
        self.token
    }

    /// Make an authorised request to the API.
    /// `duration` is the expected time required to make the request.
    pub async fn request<R, F>(&mut self, build_request: F, duration: Duration) -> Result<R>
    where
        R: serde::de::DeserializeOwned,
        F: Fn(&reqwest::Client) -> reqwest::RequestBuilder,
    {
        let token = self.get_valid_token_for(duration).await?;
        request(&crate::CLIENT, |client| {
            build_request(client).bearer_auth(token.as_str())
        })
        .await
    }

    async fn get_valid_token_for(&mut self, duration: Duration) -> Result<&AccessToken> {
        if !self.token.is_valid_for(duration) {
            self.token = std::mem::take(&mut self.token).refresh().await?;
            assert!(self.token.is_valid_for(duration))
        }
        Ok(&self.token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_request() {
        #[derive(Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
        struct Response {
            code: u16,
            description: String,
        }
        const TIME_DIFF: Duration = Duration::from_secs(5);

        let mut invalid_token = AccessToken::default();
        invalid_token.expires_at += TIME_DIFF; // preted to be valid
        let mut client = Client::new(invalid_token);

        let res: Result<Response> = client
            .request(
                |client| client.get("https://httpstat.us/200"),
                TIME_DIFF / 2,
            )
            .await;
        assert_eq!(
            res.expect("A valid response"),
            Response {
                code: 200,
                description: "OK".to_owned()
            }
        );
    }
}

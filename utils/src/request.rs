//! [`reqwest`] wrapper for consistant error handling and formatted errors
use ::reqwest as req;

/// A human readable HTTP Status code error with response body
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct StatusError {
    /// The HTTP status code/error
    pub status: req::StatusCode,
    /// The body of the response
    pub body: Option<String>,
}
impl std::error::Error for StatusError {}
impl std::fmt::Display for StatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HTTP Error: {}{}{}",
            self.status.as_str(),
            if let Some(desc) = self.status.canonical_reason() {
                ", ".to_string() + desc
            } else {
                "".to_string()
            },
            if let Some(body) = &self.body {
                "\nBody:\n".to_string() + body
            } else {
                "".to_string()
            }
        )
    }
}

/// A human readable response body deserialization error
#[derive(Debug)]
pub struct JSONError {
    /// The deserialization error
    pub error: serde_json::Error,
    /// The response body that failed to deserialize
    pub body: String,
}
impl std::error::Error for JSONError {}
impl std::fmt::Display for JSONError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "deserialization error, {} in {}", self.error, self.body)
    }
}

/// HTTP request error with human readable endpoint errors
#[derive(Debug)]
pub enum Error {
    /// A human readable HTTP Status code error with response body
    Status(StatusError),
    /// A human readable response body deserialization error
    Body(JSONError),
    /// Other reswest errors
    Reqwest(req::Error),
}
impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::Status(status) => status.to_string(),
                Error::Reqwest(req) => req.to_string(),
                Error::Body(body) => body.to_string(),
            }
        )
    }
}

/// HTTP request result
pub type Result<R> = std::result::Result<R, Error>;

/// [`reqwest`] wrapper for deserializing response and consistant error handling
pub async fn request<R, F>(client: &req::Client, build_request: F) -> Result<R>
where
    R: serde::de::DeserializeOwned,
    F: Fn(&req::Client) -> req::RequestBuilder,
{
    let res = build_request(client)
        .header(
            req::header::ACCEPT,
            req::header::HeaderValue::from_static("application/json"),
        )
        .send()
        .await
        .map_err(Error::Reqwest)?;
    let status = res.status();
    if status.is_client_error() || status.is_server_error() {
        Err(Error::Status(StatusError {
            status,
            body: res.text().await.ok(),
        }))
    } else {
        let body = res.text().await.map_err(Error::Reqwest)?;
        Ok(serde_json::from_str(&body).map_err(|error| Error::Body(JSONError { error, body }))?)
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
        #[derive(Deserialize, Debug)]
        struct Response {
            ip: std::net::IpAddr,
        }

        #[derive(Deserialize, Debug)]
        struct WrongResponse {
            #[serde(rename = "ip")]
            _ip: u128,
        }

        let res: Result<Response> = request(&req::Client::new(), |client| {
            client.get("http://ip.jsontest.com/")
        })
        .await;
        assert!(!res.expect("A valid Response").ip.to_string().is_empty());

        let res: Result<WrongResponse> = request(&req::Client::new(), |client| {
            client.get("http://ip.jsontest.com/")
        })
        .await;
        console_log!("{}", res.expect_err("An invalid response"));
    }

    #[wasm_bindgen_test]
    async fn test_request_http_status() {
        let res: Result<()> = request(&req::Client::new(), |client| {
            client.get("https://httpstat.us/418")
        })
        .await;
        let err = res.expect_err("An invalid response");
        console_log!("{err}");
        assert!(matches!(
            err,
            Error::Status(StatusError {
                status: req::StatusCode::IM_A_TEAPOT,
                body: _,
            })
        ));

        let res: Result<()> = request(&req::Client::new(), |client| {
            client.get("https://httpstat.us/404")
        })
        .await;
        let err = res.expect_err("An invalid response");
        console_log!("{err}");
        assert!(matches!(
            err,
            Error::Status(StatusError {
                status: req::StatusCode::NOT_FOUND,
                body: _,
            })
        ));
    }
}

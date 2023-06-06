use super::*;

lazy_static! {
    static ref ENDPOINT: utils::Url = crate::authorisation::ENDPOINT.with_path(["api", "token"]);
}

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
enum TokenType {
    #[default]
    Bearer,
}

use super::AuthorisationBuilder;

/// Errors for a malformed authorise callback URL
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum CallbackUrlError {
    /// The `code` query paramater is missing
    CodeMissing,
    /// The error from the `error` query paramater
    LoginError(String),
    /// The `state` query paramater doesn't match the state from the [`authorise_url()`](crate::authorisation::AuthorisationBuilder::authorise_url())
    StateMismatch(utils::Mismatch<String>),
    /// The `state` query paramater is missing
    StateMissing,
    /// The callback URL doesn't match the URL given to [`authorise_url()`](crate::authorisation::AuthorisationBuilder::authorise_url())
    UrlMismatch(utils::Mismatch<reqwest::Url>),
}
use CallbackUrlError::*;
impl std::error::Error for CallbackUrlError {}
impl std::fmt::Display for CallbackUrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CodeMissing =>
                    "the `code` query paramater is missing from the callback URL".to_owned(),
                LoginError(err) => format!("the callback URL contained an `error` query paramater: \"{err}\""),
                StateMismatch(mismatch) => format!("the `state` query paramater in the callback URL doesn't match the state from the authorize URL, {mismatch}"),
                StateMissing => "the `state` query paramater is missing from the callback URL".to_owned(),
                UrlMismatch(mismatch) => format!("the `redirect_uri` query paramater in the callback URL doesn't match the authorize URL, {mismatch}"),
            }
        )
    }
}

/// [`AccessToken::new()`] Errors
#[derive(Debug)]
pub enum AccessTokenError {
    /// Errors for a malformed authorise callback URL
    CallbackUrl(CallbackUrlError),
    /// Errors requesting an access token
    Request(utils::request::Error),
}
use instant::Duration;
use AccessTokenError::*;
impl std::error::Error for AccessTokenError {}
impl std::fmt::Display for AccessTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CallbackUrl(err) => err.to_string(),
                Request(err) => err.to_string(),
            }
        )
    }
}

impl PartialEq for AccessTokenError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::CallbackUrl(l_url), Self::CallbackUrl(r_url)) => l_url == r_url,
            (Self::Request(l_req), Self::Request(r_req)) => l_req.to_string() == r_req.to_string(),
            (_self, _other) => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

#[derive(
    Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, serde::Serialize, serde::Deserialize,
)]
struct RefreshToken {
    pub(super) token: String,
    pub(super) client_id: String,
}

/// Used to make authorised requests to the Spotify API
#[derive(
    Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, serde::Serialize, serde::Deserialize,
)]
pub struct AccessToken {
    #[serde(skip)]
    /// The access token
    pub(super) token: String,
    #[serde(skip)]
    #[serde(default = "instant::Instant::now")]
    /// The time at which this token will expire
    pub expires_at: instant::Instant,
    #[serde(with = "crate::authorisation::scopes::serialize_scopes")]
    /// [Spotify authorisation scopes](https://developer.spotify.com/documentation/general/guides/authorization/scopes/)
    /// available for this [`AccessToken`]
    pub scope: Vec<crate::authorisation::Scopes>,
    refresh_token: RefreshToken,
}

impl std::default::Default for AccessToken {
    /// An invalid access token
    fn default() -> Self {
        AccessToken {
            token: "invalid-access-token".to_owned(),
            expires_at: instant::Instant::now(),
            scope: vec![],
            refresh_token: RefreshToken {
                token: "invalid-refresh-token".to_owned(),
                client_id: "invalid-client-id".to_owned(),
            },
        }
    }
}

impl AccessToken {
    /// Uses the the provided authorised callback url to request an access token.
    pub async fn new(
        state: AuthorisationBuilder,
        callback_url: ::url::Url,
    ) -> Result<Self, AccessTokenError> {
        let query_params = callback_url
        .query_pairs()
        .collect::<std::collections::HashMap<std::borrow::Cow<'_, str>, std::borrow::Cow<'_, str>>>();

        match query_params.get("error") {
            Some(err) => Err(CallbackUrl(LoginError(err.to_string()))),
            None => Ok(()),
        }?;

        match (query_params.get("state"), state.session_state) {
            (None, _expected) => Err(CallbackUrl(StateMissing)),
            (Some(received), expected) if *received != expected => {
                Err(CallbackUrl(StateMismatch(utils::Mismatch {
                    expected,
                    received: received.to_string(),
                })))
            }
            (Some(_received), _expected) => Ok((/* state matched */)),
        }?;

        let code = query_params.get("code").ok_or(CallbackUrl(CodeMissing))?;

        if !callback_url
            .to_string()
            .starts_with(&state.callback_url.to_string())
        {
            return Err(CallbackUrl(UrlMismatch(utils::Mismatch {
                expected: (*state.callback_url).clone(),
                received: callback_url,
            })));
        }

        #[derive(
            Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, serde::Deserialize,
        )]
        struct Response {
            access_token: String,
            token_type: TokenType,
            #[serde(default, deserialize_with = "deserialize_optional_scopes")]
            scope: Option<Vec<crate::authorisation::Scopes>>,
            #[serde(deserialize_with = "deserialize_seconds")]
            expires_in: instant::Duration,
            refresh_token: String,
        }

        fn deserialize_optional_scopes<'de, D: serde::Deserializer<'de>>(
            deserializer: D,
        ) -> Result<Option<Vec<crate::authorisation::Scopes>>, D::Error> {
            struct Visitor;
            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = Option<Vec<crate::authorisation::Scopes>>;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("An optional string containing 0 or more Scopes")
                }

                fn visit_some<D: serde::Deserializer<'de>>(
                    self,
                    deserializer: D,
                ) -> Result<Self::Value, D::Error> {
                    crate::authorisation::scopes::serialize_scopes::deserialize(deserializer)
                        .map(Some)
                }

                fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                    Ok(None)
                }
            }
            deserializer.deserialize_option(Visitor)
        }

        pub fn deserialize_seconds<'de, D: serde::Deserializer<'de>>(
            deserializer: D,
        ) -> Result<instant::Duration, D::Error> {
            struct Visitor;
            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = instant::Duration;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("An integer containing 0 or more seconds")
                }

                fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
                    Ok(instant::Duration::from_secs(value))
                }
            }

            deserializer.deserialize_u64(Visitor)
        }

        let res: Response = utils::request(&crate::CLIENT, |client| {
            client
                .post(crate::authorisation::tokens::ENDPOINT.as_str())
                .form(&[
                    ("grant_type", "authorization_code"),
                    ("code", code),
                    ("redirect_uri", state.callback_url.as_str()),
                    ("client_id", &state.client_id),
                    ("code_verifier", &state.code_verifier),
                ])
        })
        .await
        .map_err(Request)?;

        Ok(AccessToken {
            token: res.access_token.to_owned(),
            expires_at: instant::Instant::now() + res.expires_in,
            scope: if let Some(scope) = res.scope {
                scope
            } else {
                state.scope
            },
            refresh_token: RefreshToken {
                token: res.refresh_token,
                client_id: state.client_id,
            },
        })
    }

    /// Refreshes the access token
    pub async fn refresh(self) -> utils::request::Result<Self> {
        #[derive(
            Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, serde::Deserialize,
        )]
        struct Response {
            access_token: String,
            token_type: TokenType,
            expires_in: std::time::Duration,
        }

        let res: Response = utils::request(&crate::CLIENT, |client| {
            client
                .post(crate::authorisation::tokens::ENDPOINT.as_str())
                .form(&[
                    ("grant_type", "refresh_token"),
                    ("refresh_token", &self.token),
                    ("client_id", &self.refresh_token.client_id),
                ])
        })
        .await?;

        Ok(AccessToken {
            token: res.access_token,
            expires_at: instant::Instant::now() + res.expires_in,
            refresh_token: self.refresh_token,
            scope: self.scope,
        })
    }

    /// Checks if the token will be valid for the given duration.
    /// Otherwise the token will expire sometime in the duration.
    pub fn is_valid_for(&self, duration: Duration) -> bool {
        match self.expires_at.checked_sub(duration) {
            Some(valid_until) => instant::Instant::now() < valid_until,
            None => false,
        }
    }

    /// get a reference to the access token
    pub fn as_str(&self) -> &str {
        &self.token
    }
}

impl std::fmt::Display for AccessToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_token_expiry() {
        use instant::{Duration, Instant};
        const TIME_DIFF: Duration = Duration::from_secs(1);

        let invalid = AccessToken::default();
        assert!(!invalid.is_valid_for(Duration::ZERO));
        assert!(!invalid.is_valid_for(TIME_DIFF));

        let expired = AccessToken {
            token: "access-token".to_string(),
            expires_at: Instant::now(),
            scope: vec![],
            refresh_token: RefreshToken {
                token: "refresh-token".to_string(),
                client_id: "client-id".to_string(),
            },
        };
        utils::delay(TIME_DIFF).await;
        assert!(!expired.is_valid_for(TIME_DIFF));
        assert!(!expired.is_valid_for(Duration::ZERO));

        let valid = AccessToken {
            token: "access-token".to_string(),
            expires_at: Instant::now() + TIME_DIFF,
            scope: vec![],
            refresh_token: RefreshToken {
                token: "refresh-token".to_string(),
                client_id: "client-id".to_string(),
            },
        };
        assert!(valid.is_valid_for(TIME_DIFF - TIME_DIFF / 2));
        assert!(valid.is_valid_for(Duration::ZERO));
        assert!(!expired.is_valid_for(TIME_DIFF * 2));
    }

    #[wasm_bindgen_test]
    async fn test_new_token() {
        let base_url = utils::Url::from_browser_location();
        //  ::parse(&format!(
        //     "{}/authorised",
        //     web_sys::window().unwrap().location().origin().unwrap()
        // ))
        // .expect("Invalid base URL");

        let builder = AuthorisationBuilder::new("id");
        let session_state = builder.session_state.clone();
        assert_eq!(
            AccessToken::new(
                builder,
                base_url
                    .clone()
                    .query_pairs_mut()
                    .append_pair("code", "some code")
                    .append_pair("state", &session_state)
                    .finish()
                    .to_owned(),
            )
            .await,
            Err(Request(utils::request::Error::Status(
                utils::request::StatusError {
                    status: reqwest::StatusCode::BAD_REQUEST,
                    body: Some(
                        r#"{"error":"invalid_client","error_description":"Invalid client"}"#
                            .to_string()
                    )
                }
            )))
        );

        let builder = AuthorisationBuilder::new("id");
        assert_eq!(
            AccessToken::new(
                builder,
                base_url
                    .clone()
                    .query_pairs_mut()
                    .append_pair("error", "some error without state")
                    .finish()
                    .to_owned(),
            )
            .await,
            Err(CallbackUrl(LoginError(
                "some error without state".to_string()
            )))
        );

        let builder = AuthorisationBuilder::new("id");
        assert_eq!(
            AccessToken::new(
                builder,
                base_url
                    .clone()
                    .query_pairs_mut()
                    .append_pair("error", "some error")
                    .append_pair("state", "untested state")
                    .finish()
                    .to_owned(),
            )
            .await,
            Err(CallbackUrl(LoginError("some error".to_string())))
        );

        let builder = AuthorisationBuilder::new("id");
        assert_eq!(
            AccessToken::new(
                builder,
                base_url
                    .clone()
                    .query_pairs_mut()
                    .append_pair("code", "code without state")
                    .finish()
                    .to_owned(),
            )
            .await,
            Err(CallbackUrl(StateMissing))
        );

        let builder = AuthorisationBuilder::new("id");
        let session_state = builder.session_state.clone();
        assert_eq!(
            AccessToken::new(
                builder,
                base_url
                    .clone()
                    .query_pairs_mut()
                    .append_pair("state", &session_state)
                    .finish()
                    .to_owned(),
            )
            .await,
            Err(CallbackUrl(CodeMissing))
        );

        let mut builder = AuthorisationBuilder::new("id");
        builder.callback_url_path(&["mismatched_url"]);
        let session_state = builder.session_state.clone();
        let callback_url = (*builder.callback_url).clone();
        let mismatch_callback_url = base_url
            .clone()
            .query_pairs_mut()
            .append_pair("code", "some code")
            .append_pair("state", &session_state)
            .finish()
            .to_owned();
        assert_eq!(
            AccessToken::new(builder, mismatch_callback_url.clone()).await,
            Err(CallbackUrl(UrlMismatch(utils::Mismatch {
                expected: callback_url,
                received: mismatch_callback_url,
            })))
        );

        let builder = AuthorisationBuilder::new("id");
        let session_state = builder.session_state.clone();
        assert_eq!(
            AccessToken::new(
                builder,
                base_url
                    .clone()
                    .query_pairs_mut()
                    .append_pair("code", "some code")
                    .append_pair("state", "mismatches state")
                    .finish()
                    .to_owned(),
            )
            .await,
            Err(CallbackUrl(StateMismatch(utils::Mismatch {
                expected: session_state,
                received: "mismatches state".to_string()
            })))
        );
    }
}

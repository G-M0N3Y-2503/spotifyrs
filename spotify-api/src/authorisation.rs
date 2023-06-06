//! Operations for getting authorised access to the spotify API
//!
//! ## Example
//! Navigate to the authorising URL
//! ```
//!  let _authorise_read_email_access_token = api::authorisation::authorise_url(
//!         &mut utils::SessionStore {},
//!         "CLIENT_ID",
//!         &[api::authorisation::Scopes::UserReadEmail],
//!         "authorised",
//!     ).await;
//! ```
//! Once called back to the app exchange the url for an access token
//! ```
//!  let _read_email_access_token = api::authorisation::AccessToken::new(
//!         &mut utils::SessionStore {},
//!         ::url::Url::parse("http://localhost/authorised?code=authorisation_code&state=authorisation_state")?,
//!     ).await;
//! ```

use ::utils::*;
use lazy_static::lazy_static;
use reqwest::Url;

mod scopes;
pub use scopes::*;
mod tokens;
pub use tokens::*;

lazy_static! {
    static ref ENDPOINT: utils::Url = Url::parse("https://accounts.spotify.com")
        .expect("Valid authorisation URL")
        .try_into()
        .expect("URL is a base URL");
}

/// Authorisation state used by [`AuthorisationBuilder::authorise_url()`](self::AuthorisationBuilder::authorise_url()) and [`AccessToken::new()`](self::AccessToken::new())
#[derive(
    Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, serde::Serialize, serde::Deserialize,
)]
pub struct AuthorisationBuilder {
    client_id: String,
    callback_url: utils::Url,
    #[serde(with = "scopes::serialize_scopes")]
    scope: Vec<Scopes>,

    /// 32 base64 encoded chars
    session_state: String,

    /// 128 base64 encoded chars
    code_verifier: String,
}

impl AuthorisationBuilder {
    /// Creates an [`AuthorisationBuilder`] with default values.
    ///
    /// ### Defaults
    /// `session_state`: 32 cryptographically random base64 encoded chars
    /// `code_verifier`: 128 cryptographically random base64 encoded chars
    /// `callback_url`: The current origin of the browser URL and '/authorised' as the path.
    pub fn new(client_id: &str) -> Result<Self, NotABaseError> {
        Ok(Self {
            client_id: client_id.to_owned(),
            session_state: random(24),
            code_verifier: random(96),
            scope: Vec::new(),
            callback_url: utils::Url::from_browser_location()?.with_path(["authorised"]),
        })
    }

    /// Set the session state.
    /// Returns unmodified if at least 1 byte isn't provided.
    pub fn session_state(&mut self, state: &[u8]) -> Result<&mut Self, &mut Self> {
        if state.is_empty() {
            Err(self)
        } else {
            self.session_state = utils::base64(state);
            Ok(self)
        }
    }

    /// Set the session state.
    /// Returns unmodified if less than 32 bytes or greater than 96 bytesis provided.
    pub fn code_verifier(&mut self, state: &[u8]) -> Result<&mut Self, &mut Self> {
        if state.len() < 32 || state.len() > 96 {
            Err(self)
        } else {
            self.code_verifier = utils::base64(state);
            Ok(self)
        }
    }

    /// Set the callback URL.
    ///
    /// [`callback_url_path()`](Self::callback_url_path()) can be used to use the current origin and just set the path.
    pub fn callback_url(&mut self, url: Url) -> Result<&mut Self, NotABaseError> {
        self.callback_url = url.try_into()?;
        Ok(self)
    }

    /// Set the callback URL to the current browser origin with the provided `callback_url_path` appended.
    ///
    /// To set the entire URL use [`callback_url()`](Self::callback_url())
    pub fn callback_url_path(
        &mut self,
        callback_url_path: &[&str],
    ) -> Result<&mut Self, NotABaseError> {
        self.callback_url = utils::Url::from_browser_location()?.with_path(callback_url_path);
        Ok(self)
    }

    /// Creates a URL for authorising an access token.
    ///
    /// Once authorised the app will be redirected to the configured `callback_url`.
    /// If `scopes` is empty, authorisation will be granted only to access publicly available information.
    pub async fn authorise_url(&mut self, scope: &[Scopes]) -> Url {
        let mut url = ENDPOINT.with_path(["authorize"]);
        let mut query_pairs = url.query_pairs_mut();
        query_pairs
            .append_pair("client_id", &self.client_id)
            .append_pair("response_type", "code")
            .append_pair("redirect_uri", self.callback_url.as_ref())
            .append_pair("state", &self.session_state)
            .append_pair("code_challenge_method", "S256")
            .append_pair(
                "code_challenge",
                &sha256(self.code_verifier.as_bytes()).await,
            );
        if !scope.is_empty() {
            self.scope = scope.to_vec();
            query_pairs.append_pair("scope", &String::from_iter(scope));
        }

        query_pairs.finish().to_owned()
    }

    /// Request an access token with the authorised callback url
    pub async fn build(self, callback_url: Url) -> Result<AccessToken, AccessTokenError> {
        AccessToken::new(self, callback_url).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[derive(Clone, Eq, PartialEq, Debug, Default)]
    struct UrlCmp {
        origin: String,
        path: String,
        query_params: std::collections::HashMap<String, String>,
    }
    impl From<Url> for UrlCmp {
        fn from(url: Url) -> Self {
            UrlCmp {
                origin: url.origin().unicode_serialization(),
                path: url.path().to_string(),
                query_params: url
                    .query_pairs()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            }
        }
    }
    #[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
    struct UrlCmpRef<'a> {
        origin: &'a str,
        path: &'a str,
        query_params: &'a [(&'a str, &'a str)],
    }
    impl From<UrlCmpRef<'_>> for UrlCmp {
        fn from(url_cmp: UrlCmpRef) -> Self {
            UrlCmp {
                origin: url_cmp.origin.to_owned(),
                path: url_cmp.path.to_owned(),
                query_params: url_cmp
                    .query_params
                    .iter()
                    .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                    .collect(),
            }
        }
    }

    #[wasm_bindgen_test]
    async fn test_authorise_url() {
        let mut builder = AuthorisationBuilder::new("id");
        let url = builder.authorise_url(&[]).await;
        let url: UrlCmp = url.into();
        let value_client_id = url.query_params.get("client_id").unwrap();
        assert_eq!(value_client_id, &builder.client_id);
        let value_redirect_uri = url.query_params.get("redirect_uri").unwrap();
        assert_eq!(value_redirect_uri, &builder.callback_url.to_string());
        let value_state = url.query_params.get("state").unwrap();
        assert_eq!(value_state, &builder.session_state);
        let value_code_challenge = url.query_params.get("code_challenge").unwrap();
        assert_eq!(
            value_code_challenge,
            &sha256(builder.code_verifier.as_bytes()).await
        );
        let expected: UrlCmp = UrlCmpRef {
            origin: "https://accounts.spotify.com",
            path: "/authorize",
            query_params: &[
                ("client_id", value_client_id),
                ("response_type", "code"),
                ("redirect_uri", value_redirect_uri),
                ("state", value_state),
                ("code_challenge_method", "S256"),
                ("code_challenge", value_code_challenge),
            ],
        }
        .into();
        assert_eq!(url, expected);

        let mut builder = AuthorisationBuilder::new("some id");
        builder
            .session_state("some state".as_bytes())
            .unwrap()
            .code_verifier("some code verifier 32 bytes long".as_bytes())
            .unwrap();
        let builder = builder.callback_url(Url::parse("http://localhost/some/url").unwrap());
        let url = builder.authorise_url(&[]).await;
        let url: UrlCmp = url.into();
        let expected: UrlCmp = UrlCmpRef {
            origin: "https://accounts.spotify.com",
            path: "/authorize",
            query_params: &[
                ("client_id", "some id"),
                ("response_type", "code"),
                ("redirect_uri", "http://localhost/some/url"),
                ("state", &base64("some state".as_bytes())),
                ("code_challenge_method", "S256"),
                (
                    "code_challenge",
                    &sha256(base64("some code verifier 32 bytes long".as_bytes()).as_bytes()).await,
                ),
            ],
        }
        .into();
        assert_eq!(url, expected);

        let mut builder = AuthorisationBuilder::new("id");
        let url = builder.authorise_url(&[Scopes::AppRemoteControl]).await;
        let url: UrlCmp = url.into();
        let value_client_id = url.query_params.get("client_id").unwrap();
        assert_eq!(value_client_id, &builder.client_id);
        let value_redirect_uri = url.query_params.get("redirect_uri").unwrap();
        assert_eq!(value_redirect_uri, &builder.callback_url.to_string());
        let value_state = url.query_params.get("state").unwrap();
        assert_eq!(value_state, &builder.session_state);
        let value_code_challenge = url.query_params.get("code_challenge").unwrap();
        assert_eq!(
            value_code_challenge,
            &sha256(builder.code_verifier.as_bytes()).await
        );
        let expected: UrlCmp = UrlCmpRef {
            origin: "https://accounts.spotify.com",
            path: "/authorize",
            query_params: &[
                ("client_id", value_client_id),
                ("response_type", "code"),
                ("redirect_uri", value_redirect_uri),
                ("state", value_state),
                ("scope", "app-remote-control"),
                ("code_challenge_method", "S256"),
                ("code_challenge", value_code_challenge),
            ],
        }
        .into();
        assert_eq!(url, expected);

        let mut builder = AuthorisationBuilder::new("id");
        let url = builder
            .authorise_url(&[Scopes::AppRemoteControl, Scopes::PlaylistModifyPrivate])
            .await;
        let url: UrlCmp = url.into();
        let value_client_id = url.query_params.get("client_id").unwrap();
        assert_eq!(value_client_id, &builder.client_id);
        let value_redirect_uri = url.query_params.get("redirect_uri").unwrap();
        assert_eq!(value_redirect_uri, &builder.callback_url.to_string());
        let value_state = url.query_params.get("state").unwrap();
        assert_eq!(value_state, &builder.session_state);
        let value_code_challenge = url.query_params.get("code_challenge").unwrap();
        assert_eq!(
            value_code_challenge,
            &sha256(builder.code_verifier.as_bytes()).await
        );
        let expected: UrlCmp = UrlCmpRef {
            origin: "https://accounts.spotify.com",
            path: "/authorize",
            query_params: &[
                ("client_id", value_client_id),
                ("response_type", "code"),
                ("redirect_uri", value_redirect_uri),
                ("state", value_state),
                ("scope", "app-remote-control playlist-modify-private"),
                ("code_challenge_method", "S256"),
                ("code_challenge", value_code_challenge),
            ],
        }
        .into();
        assert_eq!(url, expected);
    }

    #[wasm_bindgen_test]
    fn test_state_serialise_deserialize() -> serde_json::Result<()> {
        let state = AuthorisationBuilder::new("client_id");
        let state_serialised = serde_json::to_string(&state)?;
        let state_deserialised = serde_json::from_str(&state_serialised)?;
        assert_eq!(state, state_deserialised);
        Ok(())
    }

    #[wasm_bindgen_test]
    fn test_state_serialise() {
        assert_eq!(
            serde_json::to_string(&AuthorisationBuilder {
                client_id: "id".to_string(),
                session_state: "state".to_string(),
                code_verifier: "code".to_string(),
                callback_url: Url::parse(&browser_window().location().origin().unwrap())
                    .unwrap()
                    .into(),
                scope: vec![Scopes::AppRemoteControl, Scopes::PlaylistModifyPrivate]
            })
            .unwrap(),
            format!(
                r#"{{"client_id":"id","callback_url":"{}","scope":"app-remote-control playlist-modify-private","session_state":"state","code_verifier":"code"}}"#,
                Url::parse(&browser_window().location().origin().unwrap()).unwrap()
            )
        );
    }

    #[wasm_bindgen_test]
    fn test_state_deserialized() {
        assert_eq!(
            serde_json::from_str::<AuthorisationBuilder>(&format!(
                r#"{{"client_id":"id","callback_url":"{}","scope":"app-remote-control playlist-modify-private","session_state":"state","code_verifier":"code"}}"#,
                Url::parse(&browser_window().location().origin().unwrap()).unwrap()
            )).unwrap(),
            AuthorisationBuilder {
                client_id: "id".to_string(),
                session_state: "state".to_string(),
                code_verifier: "code".to_string(),
                callback_url: Url::parse(&browser_window().location().origin().unwrap()).unwrap().into(),
                scope: vec![Scopes::AppRemoteControl, Scopes::PlaylistModifyPrivate]
            }
        );
        assert_eq!(
            serde_json::from_str::<AuthorisationBuilder>(r#"{"client_id":"id","session_state":"state","code_verifier":"code","callback_url":"invalid url"}"#,
            ).map_err(|err| std::string::ToString::to_string(&err)),
            Err("invalid value: invalid url, expected A string containing a valid URL at line 1 column 93".to_string())
        );
        assert_eq!(
            serde_json::from_str::<AuthorisationBuilder>(
                r#"{"client_id":"id","session_state":"state","code_verifier":"code"}"#,
            )
            .map_err(|err| std::string::ToString::to_string(&err)),
            Err("missing field `callback_url` at line 1 column 65".to_string())
        );
        assert_eq!(
            serde_json::from_str::<AuthorisationBuilder>(&format!(
                r#"{{"client_id":"id","session_state":"state","code_verifier":"code","session_state":"state","callback_url":"{}/"}}"#,
                browser_window().location().origin().unwrap()
            )).map_err(|err| std::string::ToString::to_string(&err)),
            Err("duplicate field `session_state` at line 1 column 80".to_string())
        );
    }
}

use super::*;
use spotify::authorisation::AccessTokenError;
use std::time::Duration;

#[derive(PartialEq, Debug)]
pub enum AuthorizationError {
    NoAuthorizationState,
    Store(WebStoreError),
    AccessToken(AccessTokenError),
}
impl std::fmt::Display for AuthorizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AuthorizationError::NoAuthorizationState =>
                    "authorisation request state is missing".to_string(),
                AuthorizationError::Store(err) => err.to_string(),
                AuthorizationError::AccessToken(err) => err.to_string(),
            }
        )
    }
}
impl std::error::Error for AuthorizationError {}

#[derive(Clone, PartialEq, Debug, Default, Properties)]
pub struct Properties {
    pub on_client: Callback<spotify::Client>,
}

#[derive(PartialEq, Debug)]
pub enum Message {
    UpdateAccessToken(Result<spotify::authorisation::AccessToken, AuthorizationError>),
}

#[derive(PartialEq, Debug, Default)]
pub struct Authorised {
    access_token: Option<Result<(), AuthorizationError>>,
}

impl Component for Authorised {
    type Message = Message;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async {
            Self::Message::UpdateAccessToken(
                match SessionStore::new()
                    .remove(&app::StoreKeys::AuthorisationBuilder)
                    .map_err(AuthorizationError::Store)
                    .and_then(|store_value| {
                        store_value.ok_or(AuthorizationError::NoAuthorizationState)
                    }) {
                    Ok(auth_builder) => {
                        spotify::authorisation::AccessToken::new(auth_builder, browser_location())
                            .await
                            .map_err(AuthorizationError::AccessToken)
                    }
                    Err(err) => Err(err),
                },
            )
        });
        Default::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::UpdateAccessToken(access_token) => {
                self.access_token = Some(match access_token {
                    Ok(access_token) => {
                        ctx.props()
                            .on_client
                            .emit(spotify::Client::new(access_token));
                        Ok(())
                    }
                    Err(err) => Err(err),
                });
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let navigator = ctx
            .link()
            .navigator()
            .expect("the browser history can be accessed");
        let tmp = app::utils::RedirectCallbackBuilder::new(app::UnauthorisedRoutes::Authorised)
            .delay_for(instant::Duration::from_secs(5))
            .build()
            .start(navigator);

        match &self.access_token {
            Some(Ok(())) => {
                html! {
                    <>
                        <h1>{"Authorised successfully"}</h1>
                        <components::DelayedRedirect<app::AuthorisedRoutes>
                            delay={Duration::from_millis(2500)}
                            redirect_to={components::delayed_redirect::Location {route: app::AuthorisedRoutes::Home, description: Some("App".to_string())}}
                        />
                    </>
                }
            }
            Some(Err(err)) => {
                let (err, diagnostic, retry) = match err {
                    AuthorizationError::AccessToken(AccessTokenError::Request(err)) => (
                        "failed to request authorisation".to_string(),
                        Some(err.to_string()),
                        true,
                    ),
                    AuthorizationError::AccessToken(AccessTokenError::CallbackUrl(err)) => (
                        "the request responded with an error".to_string(),
                        Some(err.to_string()),
                        true,
                    ),
                    AuthorizationError::NoAuthorizationState => (err.to_string(), None, true),
                    AuthorizationError::Store(err) => (
                        "error using browser storage".to_string(),
                        Some(err.to_string()),
                        false,
                    ),
                };
                html! {
                    <>
                        <h1>{"Authorisation Error"}</h1>
                        <h2>{err}</h2>
                        if let Some(diagnostic) = diagnostic {
                            <p>{diagnostic}</p>
                        }
                        if retry {
                            <components::DelayedRedirect<app::UnauthorisedRoutes>
                                delay={Duration::from_secs(10)}
                                redirect_to={components::delayed_redirect::Location {route: app::UnauthorisedRoutes::Home, description: Some("login to try again".to_string())}}
                            />
                        }
                    </>
                }
            }
            None => html! {
                <h1>{"Authorising..."}</h1>
            },
        }
    }
}

use super::*;
use ::url::Url;
use utils::*;

#[derive(Clone, PartialEq, Debug, Default, Properties)]
pub struct Properties {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub scope: Vec<spotify::authorisation::Scopes>,
    #[prop_or(html!("Authorise"))]
    pub button_inner_html: Html,
    pub client_id: Option<String>,
    pub notification_host: NodeRef,
}

#[derive(PartialEq, Debug)]
pub enum Message {
    UpdateUrl(Result<Url, UrlError>),
    IsLoading,
}

#[derive(PartialEq, Debug)]
pub enum UrlError {
    WebStore(WebStoreError),
    InvalidUrl(NotABaseError),
}
impl std::error::Error for UrlError {}
impl std::fmt::Display for UrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UrlError::WebStore(err) => err.to_string(),
                UrlError::InvalidUrl(err) => err.to_string(),
            }
        )
    }
}

#[derive(Debug, Default)]
pub struct AuthorisationForm {
    query_param_error: Option<yew_router::history::HistoryError>,
    auth_url: Option<Result<Url, UrlError>>,
    loading: bool,
}

async fn new_url(client_id: String, scope: Vec<spotify::authorisation::Scopes>) -> Message {
    match spotify::authorisation::AuthorisationBuilder::new(&client_id) {
        Ok(mut builder) => {
            let url = builder.authorise_url(&scope).await;
            Message::UpdateUrl(
                match SessionStore::new().insert(app::StoreKeys::AuthorisationBuilder, builder) {
                    Ok(_) => Ok(url),
                    Err(err) => Err(UrlError::WebStore(err)),
                },
            )
        }
        Err(err) => Message::UpdateUrl(Err(UrlError::InvalidUrl(err))),
    }
}

impl Component for AuthorisationForm {
    type Message = Message;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let mut ret = Self::default();
        if let Some(client_id) = ctx.props().client_id.clone() {
            ret.loading = true;
            ctx.link()
                .send_future(new_url(client_id, ctx.props().scope.clone()));
        } else if let Some(location) = ctx.link().location() {
            if let Some(client_id) = match location.query::<Vec<(String, String)>>() {
                Ok(params) => params
                    .into_iter()
                    .find_map(|(name, value)| (name == "client_id").then_some(value)),
                Err(err) => {
                    ret.query_param_error = Some(err);
                    None
                }
            } {
                ret.loading = true;
                ctx.link()
                    .send_future(new_url(client_id, ctx.props().scope.clone()));
            }
        }
        ret
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::UpdateUrl(url) => {
                self.auth_url = Some(url);
                self.loading = false;
                true
            }
            Message::IsLoading => {
                self.loading = true;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let scope = ctx.props().scope.clone();
        let link = ctx.link().clone();
        let update_url =
            <&Context<AuthorisationForm>>::clone(&ctx)
                .link()
                .callback(move |event: Event| {
                    link.send_future(new_url(
                        event
                            .target_unchecked_into::<web_sys::HtmlInputElement>()
                            .value(),
                        scope.clone(),
                    ));
                    Message::IsLoading
                });

        let notification_host = ctx.props().notification_host.clone();
        let (auth_url, error) = match &self.auth_url {
            Some(Ok(auth_url)) => (Some(auth_url), None),
            Some(Err(error)) => (None, Some(error)),
            None => (None, None),
        };

        let mut class = ctx.props().class.clone();
        class.push("box".to_owned());
        class.push("section".to_owned());

        html! {
            <>
                if let Some(query_param_error) = &self.query_param_error {
                    <notification::Notification
                        host={notification_host.clone()}
                        class="is-warning"
                        open=true
                    >
                        <p>{"Could not parse the query paramaters in the URL of this page. Bookmarking a client ID URL will not work."}</p>
                        <p>{format!("Error: {query_param_error}")}</p>
                    </notification::Notification>
                }
                <form action={auth_url.map(|auth_url| auth_url.origin().ascii_serialization() + auth_url.path())} method="get" {class}>
                    <div class="field level">
                        <h1 class="level-item">{"Login"}</h1>
                    </div>
                    if ctx.props().client_id.is_none() {
                        <div class="field level">
                            <notification::InfoTrigger
                                class="level-item"
                                notification_explicit_dismissal=true
                                notification_host={notification_host.clone()}
                                notification_content={html! {
                                    <>
                                        <p>
                                            {"While this application might be fit for purpose, It's still very much in development. "}
                                            {"This means, to use this application you will have to get your own Client ID, "}
                                            <a target="_blank" href="https://developer.spotify.com/documentation/general/guides/authorization/app-settings/">
                                                {"You can use this guide to create one. "}
                                            </a>
                                            {"When requested, the \"Redirect URI\" will need to be set to: "}
                                            <code>{format!("{}authorised", utils::Url::from_browser_location().map_or("".to_owned(), |url| url.to_string()))}</code>
                                        </p>
                                    </>
                                }}
                            >
                                <input type="text" form="none" class="input" placeholder="Client ID" required=true minlength=1 onchange={update_url}
                                    value={
                                        auth_url
                                            .and_then(|auth_url| {
                                                auth_url
                                                    .query_pairs()
                                                    .find(|(name, _value)| name == "client_id")
                                                    .map(|(_name, value)| value.to_string())
                                            })
                                    }
                                />
                            </notification::InfoTrigger>
                        </div>
                    }
                    {
                        auth_url.map(|auth_url| {
                            auth_url
                                .query_pairs()
                                .map(|(name, value)| html! {<input type="hidden" name={name.to_string()} value={value.to_string()} />})
                                .collect::<Html>()
                        })
                    }
                    <div class="field level">
                        <notification::ErrorTrigger
                            class={if ctx.props().client_id.is_some() {"level-item"} else { "level-item level-left" }}
                            trigger_visible={error.is_none()}
                            balanced_trigger={ctx.props().client_id.is_some()}
                            notification_host={notification_host.clone()}
                            notification_content={
                                error.map(|error| html! { <p>{error}</p> }).unwrap_or_default()
                            }
                        >
                            <button
                                class={if self.loading {"button is-loading is-primary"} else {"button is-primary"}}
                                disabled={self.loading || error.is_some() || auth_url.is_none()}
                            >
                                {ctx.props().button_inner_html.clone()}
                            </button>
                        </notification::ErrorTrigger>
                    </div>
                </form>
            </>
        }
    }
}

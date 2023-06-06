use super::pages;
use std::time::Duration;
use yew::prelude::*;
use yew_router::prelude::*;

pub(crate) mod utils;

#[derive(strum_macros::Display, serde::Serialize, serde::Deserialize)]
pub enum StoreKeys {
    AccessToken,
    AuthorisationBuilder,
    ClientId,
}

#[derive(Clone, Routable, PartialEq, Eq)]
pub enum UnauthorisedRoutes {
    #[at("/")]
    Home,
    #[at("/authorised")]
    Authorised,
}

#[derive(Clone, Routable, PartialEq, Eq)]
pub enum AuthorisedRoutes {
    #[at("/")]
    Home,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Message {
    NewClient(spotify::Client),
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct App {
    client: Option<spotify::Client>,
}

impl Component for App {
    type Message = Message;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::NewClient(client) => {
                self.client = Some(client);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_client = ctx.link().callback(Message::NewClient);
        html! {
            <BrowserRouter>
                // if let Some(client) = self.client.clone() {
                //     <Switch<AuthorisedRoutes> render={move |routes| -> Html {
                //         match routes {
                //             AuthorisedRoutes::Home => html! {
                //                 <p>{format!("{client:?}")}</p>
                //             }
                //         }
                //     }} />
                // } else {
                    <Switch<UnauthorisedRoutes> render={move |routes| -> Html {
                        match routes {
                            UnauthorisedRoutes::Home => html! {
                                <pages::Login />
                            },
                            UnauthorisedRoutes::Authorised => html! {
                                <pages::Authorised on_client={on_client.clone()} />
                            },
                        }
                    }} />
                // }
            </BrowserRouter>
        }
    }
}

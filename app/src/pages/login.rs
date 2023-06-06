use super::*;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Login {
    client_id: Option<String>,
}

impl Component for Login {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            client_id: option_env!("CLIENT_ID").map(ToString::to_string),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let notification_host = NodeRef::default();
        html! {
            <>
                <components::notification::Drawer host={notification_host.clone()} />
                <components::notification::Timeout
                    host={notification_host.clone()}
                    timeout={instant::Duration::from_secs(5)}
                >
                    <p>{"test text for the nest notification"}</p>
                </components::notification::Timeout>
                <div class="columns is-mobile is-centered is-vcentered" style="position: relative; top: -100%; height: 100%; margin: unset">
                    <div class="column is-one-third-fullhd is-two-fifths-widescreen is-half-desktop is-three-fifths-tablet is-full-mobile">
                        <h1 class="title is-1 has-text-centered">{"Spotifyrs"}</h1>
                        <components::authorisation::AuthorisationForm
                            notification_host={notification_host.clone()}
                            client_id={self.client_id.clone()}
                            button_inner_html={ html!("Sign in with Spotify")}
                        />
                    </div>
                </div>
            </>
        }
    }
}

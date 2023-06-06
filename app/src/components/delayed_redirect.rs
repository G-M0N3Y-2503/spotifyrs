use super::*;
use std::{fmt::Debug, time::Duration};
use utils::DelayedFn;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Location<R: Routable> {
    pub route: R,
    pub description: Option<String>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Properties)]
pub struct Properties<R: Routable> {
    pub delay: Duration,
    pub redirect_to: Location<R>,
    #[prop_or(true)]
    pub no_history: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Message<R: Routable> {
    Redirect(R),
    Cancel,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct DelayedRedirect<R: Routable> {
    redirect_callback: DelayedFn,
    redirect_to: Option<R>,
}

impl<R: Routable + 'static> Component for DelayedRedirect<R> {
    type Message = Message<R>;
    type Properties = Properties<R>;

    fn create(ctx: &Context<Self>) -> Self {
        let redirect_to = ctx.props().redirect_to.clone();
        let redirect_callback_ctx = ctx.link().clone();
        Self {
            redirect_to: None,
            redirect_callback: DelayedFn::new(
                move || {
                    redirect_callback_ctx.send_message(Message::Redirect(redirect_to.route.clone()))
                },
                ctx.props().delay,
            ),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Redirect(route) => {
                self.redirect_to = Some(route);
                true
            }
            Message::Cancel => {
                self.redirect_callback.stop();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_click = ctx.link().callback(|_| Message::Cancel);
        if let (Some(route), replace) = (self.redirect_to.clone(), ctx.props().no_history) {
            if let Some(nav) = ctx.link().navigator() {
                if replace {
                    nav.replace(&route)
                } else {
                    nav.push(&route)
                }
            } else {
                log::error!(
                    "Could not redirect{} as the browser history could not be accessed",
                    if let Some(description) = &ctx.props().redirect_to.description {
                        format!(" to {description}")
                    } else {
                        "".to_owned()
                    }
                );
            }
        }
        html! {
            <>
                <button onclick={on_click} class="delete" />
                {format!(" Redirecting{} in {} seconds...", if let Some(description) = &ctx.props().redirect_to.description {format!(" to {description}")} else {"".to_owned()}, ctx.props().delay.as_secs_f64())}
            </>
        }
    }
}

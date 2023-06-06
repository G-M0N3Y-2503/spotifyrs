use super::*;
use ::utils::DelayedFn;
use instant::Instant;

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct RedirectCallback {
    pub estimated_execution_time: Instant,
    callback: DelayedFn,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct RedirectCallbackBuilder<R: Routable> {
    delay: Duration,
    redirect_to: R,
    no_history: bool,
}

impl<Route: Routable> RedirectCallbackBuilder<Route> {
    pub fn new(redirect_to: Route) -> Self {
        Self {
            delay: Duration::ZERO,
            redirect_to,
            no_history: false,
        }
    }

    /// Prevents route being added to the browser history
    pub fn no_history(&mut self) -> &mut Self {
        self.no_history = true;
        self
    }

    pub fn delay_for(&mut self, duration: Duration) -> &mut Self {
        self.delay = duration;
        self
    }

    pub fn build(&self) -> Self {
        (*self).clone()
    }

    pub fn start(self, navigator: Navigator) -> RedirectCallback {
        let redirect_to_string = self.redirect_to.to_path();
        RedirectCallback {
            estimated_execution_time: Instant::now() + self.delay,
            callback: DelayedFn::new(
                move || {
                    let route: Route = Routable::recognize(redirect_to_string.as_str())
                        .expect("deserialised route should be recognised");
                    if self.no_history {
                        navigator.replace(&route)
                    } else {
                        navigator.push(&route)
                    }
                },
                self.delay,
            ),
        }
    }
}

impl RedirectCallback {
    pub fn cancel(&mut self) {
        self.callback.stop()
    }
}

use super::*;
use instant::Duration;
use utils::*;

#[derive(Clone, PartialEq, Debug, Properties)]
pub struct Properties {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub style: String,
    pub timeout: Duration,
    pub host: NodeRef,
    pub children: Children,
}

pub struct Timeout {
    dialog_ref: NodeRef,
    close_dialog: std::rc::Rc<std::cell::RefCell<DelayedFn>>,
}

impl Component for Timeout {
    type Message = ();
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let dialog_ref: NodeRef = Default::default();
        Self {
            dialog_ref: dialog_ref.clone(),
            close_dialog: {
                let dialog_ref = dialog_ref.clone();
                std::cell::RefCell::new(DelayedFn::new(
                    move || {
                        cast_dialog(&dialog_ref).close();
                    },
                    ctx.props().timeout,
                ))
                .into()
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Properties {
            class,
            style,
            timeout,
            host,
            children,
        } = ctx.props();

        let close_dialog_ref = self.close_dialog.clone();
        let onclose = move |_| match close_dialog_ref.try_borrow_mut() {
            Ok(mut close_dialog_ref) => close_dialog_ref.stop(),
            Err(err) => log::error!("Could not cancel timeout for notification: {err}"),
        };

        let notification_open = self
            .close_dialog
            .try_borrow()
            .map(|close_dialog| !close_dialog.is_finished());
        html! {
            if let Ok(true) = notification_open {
                <Notification class={class.clone()} style={style.clone()} dialog={&self.dialog_ref} host={host} open=true {onclose} >
                    {children.clone()}
                    <progress
                        class={classes!(
                            "timeout-progress",
                            "is-small",
                            "is-loading",
                            (*class).clone()
                        )}
                        style={format!(
                            concat!(
                                "animation-duration: {}ms;",
                                "margin-top: 1.25em;"
                            ),
                            timeout.as_millis()
                        )}
                    />
                </Notification>
            }
        }
    }
}

use super::*;

mod drawer;
pub use drawer::*;
mod timeout;
pub use timeout::*;
mod trigger;
pub use trigger::*;

fn cast_dialog(node_ref: &NodeRef) -> web_sys::HtmlDialogElement {
    node_ref
        .cast::<web_sys::HtmlDialogElement>()
        .expect("Node reference should refer to a dialog that exists")
}

#[derive(Clone, PartialEq, Debug, Properties)]
pub struct NotificationProperties {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub style: String,
    #[prop_or_default]
    pub open: bool,
    #[prop_or_default]
    pub onclose: Option<Callback<web_sys::Event>>,
    #[prop_or_default]
    pub oncancel: Option<Callback<web_sys::Event>>,
    #[prop_or_default]
    pub dialog: NodeRef,
    pub host: NodeRef,
    pub children: Children,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Notification {}
impl Component for Notification {
    type Message = ();
    type Properties = NotificationProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut class = ctx.props().class.clone();
        class.push("notification".to_owned());

        if let Some(host) = ctx.props().host.cast::<web_sys::Element>() {
            create_portal(
                html! {
                    <dialog
                        ref={ctx.props().dialog.clone()}
                        class={class.clone()}
                        style={"padding: unset; z-index: 100"}
                        open={ctx.props().open}
                        oncancel={ctx.props().oncancel.clone()}
                        onclose={ctx.props().onclose.clone()}
                    >
                        <form method="dialog" class={class.clone()} >
                            <button class="delete" />
                            {ctx.props().children.clone()}
                        </form>
                    </dialog>
                },
                host,
            )
        } else {
            log::error!("No host element exists for dialog");
            Default::default()
        }
    }
}

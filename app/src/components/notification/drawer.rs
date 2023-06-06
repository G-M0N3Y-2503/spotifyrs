use super::*;

#[derive(Clone, PartialEq, Debug, Default, Properties)]
pub struct Properties {
    pub host: NodeRef,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Drawer {}
impl Component for Drawer {
    type Message = ();
    type Properties = Properties;

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="columns is-vcentered is-mobile" style="height: 100%; width: 100%; margin: unset;" >
                <div class="column container is-max-desktop" style="min-height: 30%;" ref={ctx.props().host.clone()} />
            </div>
        }
    }
}

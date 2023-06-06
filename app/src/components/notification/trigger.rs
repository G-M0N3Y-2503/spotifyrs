use super::*;

#[derive(Clone, PartialEq, Debug, Default, Properties)]
pub struct TriggerProperties {
    #[prop_or_default]
    pub class: Classes,

    #[prop_or_default]
    pub notification_explicit_dismissal: bool,
    pub notification_host: NodeRef,
    pub notification_dialog: Html,

    pub trigger: Html,
    #[prop_or_default]
    pub trigger_visible: bool,
    /// Adds an invisible trigger before the children to balance the one after
    #[prop_or(false)]
    pub balanced_trigger: bool,

    pub children: Children,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Trigger {
    dialog_ref: NodeRef,
}

impl Component for Trigger {
    type Message = ();
    type Properties = TriggerProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        macro_rules! get_tag_element {
            ($html:expr) => {
                if let yew::virtual_dom::VNode::VTag(tag) = $html {
                    Some(tag)
                } else {
                    log::error!("{:?}, must be a HTML tag element", $html);
                    None
                }
            };
        }

        macro_rules! find_attribute {
            ($attribute:literal, $element:expr) => {
                $element
                    .attributes
                    .iter()
                    .find_map(|(name, value)| (name == $attribute).then_some(value.to_owned()))
            };
        }

        let (trigger_tag, trigger_children, mut trigger_classes) =
            get_tag_element!(&ctx.props().trigger)
                .map(|trigger| {
                    (
                        trigger.tag().to_owned(),
                        trigger.children().clone(),
                        classes!(find_attribute!("class", trigger)),
                    )
                })
                .unwrap_or_else(|| {
                    let mut v_list = yew::virtual_dom::VList::new();
                    v_list.add_child(html! { "❗" });
                    ("p".to_owned(), v_list, Default::default())
                });
        trigger_classes.push("is-unselectable is-clickable".to_owned());

        let (notification_children, notification_classes) =
            get_tag_element!(&ctx.props().notification_dialog)
                .map(|notification| {
                    (
                        notification.children().clone(),
                        classes!(find_attribute!("class", notification)),
                    )
                })
                .unwrap_or_else(|| {
                    let mut v_list = yew::virtual_dom::VList::new();
                    v_list.add_child(html! { <p>{ "invalid notification element provided" }</p> });
                    (v_list, "is-danger".into())
                });

        let onclick = {
            let dialog_ref = self.dialog_ref.clone();
            move |_| cast_dialog(&dialog_ref).show()
        };

        let onmouseenter = {
            let dialog_ref = self.dialog_ref.clone();
            move |_| cast_dialog(&dialog_ref).show()
        };

        let onmouseleave = if ctx.props().notification_explicit_dismissal {
            None
        } else {
            let dialog_ref = self.dialog_ref.clone();
            Some(move |_| cast_dialog(&dialog_ref).close())
        };

        let mut class = ctx.props().class.clone();
        class.push("level-item".to_owned());

        let hidden_trigger = html! {
            <@{trigger_tag.clone()}
                class={
                    let mut hidden_trigger_classes = trigger_classes
                        .clone()
                        .into_iter()
                        .filter(|class| class != "is-clickable")
                        .collect::<Classes>();
                    hidden_trigger_classes.push("is-invisible");
                    hidden_trigger_classes
                }
            >
                {trigger_children.clone()}
            </@>
        };

        html! {
            <span {class}>
                if ctx.props().balanced_trigger {
                    {hidden_trigger.clone()}
                }
                {ctx.props().children.clone()}
                if !ctx.props().trigger_visible {
                    <@{trigger_tag} {onclick} {onmouseenter} {onmouseleave} class={trigger_classes}>
                        {trigger_children}
                    </@>
                    <Notification dialog={self.dialog_ref.clone()} host={ctx.props().notification_host.clone()} class={notification_classes} >
                        {notification_children}
                    </Notification>
                } else {
                    {hidden_trigger}
                }
            </span>
        }
    }
}

macro_rules! icon_notification_trigger {
    ($component:ident, $component_properties:ident, $notification_classes:literal, $trigger_icon:literal) => {
        #[allow(dead_code)]
        #[derive(Clone, PartialEq, Debug, Default, Properties)]
        pub struct $component_properties {
            #[prop_or_default]
            pub class: Classes,

            #[prop_or_default]
            pub notification_explicit_dismissal: bool,
            pub notification_host: NodeRef,
            pub notification_content: Html,

            #[prop_or_default]
            pub trigger_visible: bool,
            /// Adds an invisible trigger before the children to balance the one after
            #[prop_or(false)]
            pub balanced_trigger: bool,

            #[prop_or_default]
            pub children: Children,
        }

        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
        pub struct $component();
        impl Component for $component {
            type Message = ();
            type Properties = $component_properties;

            fn create(_ctx: &Context<Self>) -> Self {
                Default::default()
            }

            fn view(&self, ctx: &Context<Self>) -> Html {
                html! {
                    <Trigger
                        class={ctx.props().class.clone()}
                        notification_explicit_dismissal={ctx.props().notification_explicit_dismissal}
                        notification_host={ctx.props().notification_host.clone()}
                        notification_dialog={html! {
                            <dialog class={$notification_classes}>
                                {ctx.props().notification_content.clone()}
                            </dialog>
                        }}
                        balanced_trigger={ctx.props().balanced_trigger}
                        trigger_visible={ctx.props().trigger_visible}
                        trigger={html! {
                            <p class={"icon is-large"}>{$trigger_icon}</p>
                        }}
                    >
                        {ctx.props().children.clone()}
                    </Trigger>
                }
            }
        }
    };
}

icon_notification_trigger!(ErrorTrigger, ErrorTriggerProperties, "is-danger", "❗");
icon_notification_trigger!(WarningTrigger, WarningTriggerProperties, "is-warning", "⚠️");
icon_notification_trigger!(InfoTrigger, InfoTriggerProperties, "is-info", "❔");

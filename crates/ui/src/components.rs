use rustweb_core::{Component, Context, Props, RenderError, VNode};
use rustweb_macro::html;

#[derive(Clone, PartialEq, Debug, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Ghost,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ButtonProps {
    pub label: String,
    pub variant: ButtonVariant,
    pub disabled: bool,
    pub loading: bool,
    pub aria_label: Option<String>,
    pub children: Vec<VNode>,
}

impl Props for ButtonProps {
    fn validate(&self) -> Result<(), rustweb_core::PropsError> {
        if self.label.trim().is_empty() && self.children.is_empty() && self.aria_label.is_none() {
            return Err(rustweb_core::PropsError::Invalid {
                component: "Button".into(),
                prop: "label".into(),
                hint: "provide label, children, or aria_label".into(),
            });
        }
        Ok(())
    }
}

pub struct Button;

impl Component for Button {
    type Props = ButtonProps;
    type Msg = ();

    fn create(_ctx: &Context<Self>) -> Result<Self, rustweb_core::ComponentError> {
        Ok(Self)
    }

    fn update(
        &mut self,
        _ctx: &Context<Self>,
        _msg: Self::Msg,
    ) -> Result<rustweb_core::Cmd<Self>, rustweb_core::ComponentError> {
        Ok(rustweb_core::Cmd::None)
    }

    fn view(&self, ctx: &Context<Self>) -> Result<VNode, RenderError> {
        let p = &ctx.props;
        let variant_class = match p.variant {
            ButtonVariant::Primary => "rw-btn rw-btn--primary",
            ButtonVariant::Secondary => "rw-btn rw-btn--secondary",
            ButtonVariant::Ghost => "rw-btn rw-btn--ghost",
        };
        let aria = p.aria_label.clone().unwrap_or_default();
        let disabled = p.disabled || p.loading;
        if disabled {
            if aria.is_empty() {
                Ok(html! {
                    <button class={variant_class} disabled={true} aria-busy={p.loading} aria-label={p.label.clone()}>
                        { p.children.clone() }
                        { p.label.clone() }
                    </button>
                })
            } else {
                Ok(html! {
                    <button class={variant_class} disabled={true} aria-busy={p.loading} aria-label={aria}>
                        { p.children.clone() }
                        { p.label.clone() }
                    </button>
                })
            }
        } else if aria.is_empty() {
            Ok(html! {
                <button class={variant_class} aria-busy={p.loading} aria-label={p.label.clone()}>
                    { p.children.clone() }
                    { p.label.clone() }
                </button>
            })
        } else {
            Ok(html! {
                <button class={variant_class} aria-busy={p.loading} aria-label={aria}>
                    { p.children.clone() }
                    { p.label.clone() }
                </button>
            })
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct InputProps {
    pub value: String,
    pub placeholder: Option<String>,
    pub label: Option<String>,
    pub error: Option<String>,
    pub disabled: bool,
    pub id: String,
    pub children: Vec<VNode>,
}

impl Props for InputProps {
    fn validate(&self) -> Result<(), rustweb_core::PropsError> {
        if self.id.trim().is_empty() {
            return Err(rustweb_core::PropsError::Invalid {
                component: "Input".into(),
                prop: "id".into(),
                hint: "id is required for label association".into(),
            });
        }
        Ok(())
    }
}

pub struct Input;

impl Component for Input {
    type Props = InputProps;
    type Msg = ();

    fn create(_ctx: &Context<Self>) -> Result<Self, rustweb_core::ComponentError> {
        Ok(Self)
    }

    fn update(
        &mut self,
        _ctx: &Context<Self>,
        _msg: Self::Msg,
    ) -> Result<rustweb_core::Cmd<Self>, rustweb_core::ComponentError> {
        Ok(rustweb_core::Cmd::None)
    }

    fn view(&self, ctx: &Context<Self>) -> Result<VNode, RenderError> {
        let p = &ctx.props;
        let described = p.error.is_some();
        let error_id = format!("{}-error", p.id);
        if let Some(label) = &p.label {
            if described {
                if let Some(ph) = &p.placeholder {
                    Ok(html! {
                        <div class="rw-field">
                            <label for={p.id.clone()}>{ label.clone() }</label>
                            <input id={p.id.clone()} value={p.value.clone()} placeholder={ph.clone()} disabled={p.disabled} aria-invalid={true} aria-describedby={error_id.clone()} />
                            <span id={error_id} role="alert">{ p.error.clone().unwrap() }</span>
                        </div>
                    })
                } else {
                    Ok(html! {
                        <div class="rw-field">
                            <label for={p.id.clone()}>{ label.clone() }</label>
                            <input id={p.id.clone()} value={p.value.clone()} disabled={p.disabled} aria-invalid={true} aria-describedby={error_id.clone()} />
                            <span id={error_id} role="alert">{ p.error.clone().unwrap() }</span>
                        </div>
                    })
                }
            } else if let Some(ph) = &p.placeholder {
                Ok(html! {
                    <div class="rw-field">
                        <label for={p.id.clone()}>{ label.clone() }</label>
                        <input id={p.id.clone()} value={p.value.clone()} placeholder={ph.clone()} disabled={p.disabled} />
                    </div>
                })
            } else {
                Ok(html! {
                    <div class="rw-field">
                        <label for={p.id.clone()}>{ label.clone() }</label>
                        <input id={p.id.clone()} value={p.value.clone()} disabled={p.disabled} />
                    </div>
                })
            }
        } else if described {
            if let Some(ph) = &p.placeholder {
                Ok(html! {
                    <div class="rw-field">
                        <input id={p.id.clone()} value={p.value.clone()} placeholder={ph.clone()} disabled={p.disabled} aria-invalid={true} aria-describedby={error_id.clone()} aria-label={p.id.clone()} />
                        <span id={error_id} role="alert">{ p.error.clone().unwrap() }</span>
                    </div>
                })
            } else {
                Ok(html! {
                    <div class="rw-field">
                        <input id={p.id.clone()} value={p.value.clone()} disabled={p.disabled} aria-invalid={true} aria-describedby={error_id.clone()} aria-label={p.id.clone()} />
                        <span id={error_id} role="alert">{ p.error.clone().unwrap() }</span>
                    </div>
                })
            }
        } else if let Some(ph) = &p.placeholder {
            Ok(html! {
                <div class="rw-field">
                    <input id={p.id.clone()} value={p.value.clone()} placeholder={ph.clone()} disabled={p.disabled} aria-label={p.id.clone()} />
                </div>
            })
        } else {
            Ok(html! {
                <div class="rw-field">
                    <input id={p.id.clone()} value={p.value.clone()} disabled={p.disabled} aria-label={p.id.clone()} />
                </div>
            })
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct DialogProps {
    pub open: bool,
    pub title: String,
    pub children: Vec<VNode>,
}

impl Props for DialogProps {
    fn validate(&self) -> Result<(), rustweb_core::PropsError> {
        if self.title.trim().is_empty() {
            return Err(rustweb_core::PropsError::Invalid {
                component: "Dialog".into(),
                prop: "title".into(),
                hint: "title is required for accessible dialog".into(),
            });
        }
        Ok(())
    }
}

pub struct Dialog;

impl Component for Dialog {
    type Props = DialogProps;
    type Msg = ();

    fn create(_ctx: &Context<Self>) -> Result<Self, rustweb_core::ComponentError> {
        Ok(Self)
    }

    fn update(
        &mut self,
        _ctx: &Context<Self>,
        _msg: Self::Msg,
    ) -> Result<rustweb_core::Cmd<Self>, rustweb_core::ComponentError> {
        Ok(rustweb_core::Cmd::None)
    }

    fn view(&self, ctx: &Context<Self>) -> Result<VNode, RenderError> {
        let p = &ctx.props;
        if !p.open {
            return Ok(VNode::Empty);
        }
        Ok(html! {
            <div class="rw-dialog-backdrop" role="presentation">
                <div class="rw-dialog" role="dialog" aria-modal="true" aria-label={p.title.clone()}>
                    <div class="rw-dialog__header">
                        <h2 id="rw-dialog-title">{ p.title.clone() }</h2>
                    </div>
                    <div class="rw-dialog__body">
                        { p.children.clone() }
                    </div>
                </div>
            </div>
        })
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct TabsProps {
    pub tabs: Vec<String>,
    pub selected: usize,
    pub children: Vec<VNode>,
}

impl Props for TabsProps {
    fn validate(&self) -> Result<(), rustweb_core::PropsError> {
        if self.tabs.is_empty() {
            return Err(rustweb_core::PropsError::Invalid {
                component: "Tabs".into(),
                prop: "tabs".into(),
                hint: "at least one tab required".into(),
            });
        }
        if self.selected >= self.tabs.len() {
            return Err(rustweb_core::PropsError::Invalid {
                component: "Tabs".into(),
                prop: "selected".into(),
                hint: "selected index out of bounds".into(),
            });
        }
        Ok(())
    }
}

pub struct Tabs;

impl Component for Tabs {
    type Props = TabsProps;
    type Msg = ();

    fn create(_ctx: &Context<Self>) -> Result<Self, rustweb_core::ComponentError> {
        Ok(Self)
    }

    fn update(
        &mut self,
        _ctx: &Context<Self>,
        _msg: Self::Msg,
    ) -> Result<rustweb_core::Cmd<Self>, rustweb_core::ComponentError> {
        Ok(rustweb_core::Cmd::None)
    }

    fn view(&self, ctx: &Context<Self>) -> Result<VNode, RenderError> {
        let p = &ctx.props;
        let tablist: Vec<VNode> = p
            .tabs
            .iter()
            .enumerate()
            .map(|(i, label)| {
                let selected = i == p.selected;
                html! {
                    <button role="tab" aria-selected={selected} tabindex={if selected { 0 } else { -1 }}>{ label.clone() }</button>
                }
            })
            .collect();
        let panel = p.children.get(p.selected).cloned().unwrap_or(VNode::Empty);
        Ok(html! {
            <div class="rw-tabs">
                <div role="tablist">{ tablist }</div>
                <div role="tabpanel">{ panel }</div>
            </div>
        })
    }
}

pub fn render_button(props: ButtonProps) -> VNode {
    let link = rustweb_core::Link::new(|_: ()| {});
    let ctx = Context::new(props.clone(), link, Default::default());
    Button::create(&ctx)
        .and_then(|s| Ok(s.view(&ctx)?))
        .unwrap_or(VNode::Empty)
}

pub fn render_input(props: InputProps) -> VNode {
    let link = rustweb_core::Link::new(|_: ()| {});
    let ctx = Context::new(props.clone(), link, Default::default());
    Input::create(&ctx)
        .and_then(|s| Ok(s.view(&ctx)?))
        .unwrap_or(VNode::Empty)
}

pub fn render_dialog(props: DialogProps) -> VNode {
    let link = rustweb_core::Link::new(|_: ()| {});
    let ctx = Context::new(props.clone(), link, Default::default());
    Dialog::create(&ctx)
        .and_then(|s| Ok(s.view(&ctx)?))
        .unwrap_or(VNode::Empty)
}

pub fn render_tabs(props: TabsProps) -> VNode {
    let link = rustweb_core::Link::new(|_: ()| {});
    let ctx = Context::new(props.clone(), link, Default::default());
    Tabs::create(&ctx)
        .and_then(|s| Ok(s.view(&ctx)?))
        .unwrap_or(VNode::Empty)
}

use crate::context::ContextMap;
use crate::error::{ComponentError, PropsError, RenderError};
use crate::props::Props;
use crate::vnode::VNode;
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum Cmd<C: Component> {
    None,

    Render,

    Msg(C::Msg),

    Batch(Vec<Cmd<C>>),
}

#[allow(clippy::derivable_impls)]
impl<C: Component> Default for Cmd<C> {
    fn default() -> Self {
        Cmd::None
    }
}

pub struct Link<C: Component> {
    sender: std::rc::Rc<dyn Fn(C::Msg)>,
}

impl<C: Component> Clone for Link<C> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<C: Component> std::fmt::Debug for Link<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Link").finish_non_exhaustive()
    }
}

impl<C: Component> Link<C> {
    pub fn new(sender: impl Fn(C::Msg) + 'static) -> Self {
        Self {
            sender: std::rc::Rc::new(sender),
        }
    }

    pub fn send(&self, msg: C::Msg) {
        (self.sender)(msg);
    }
}

pub struct Context<C: Component> {
    pub props: C::Props,

    pub link: Link<C>,

    pub contexts: ContextMap,

    pub depth: usize,

    _marker: PhantomData<C>,
}

impl<C: Component> Clone for Context<C> {
    fn clone(&self) -> Self {
        Self {
            props: self.props.clone(),
            link: self.link.clone(),
            contexts: self.contexts.clone(),
            depth: self.depth,
            _marker: PhantomData,
        }
    }
}

impl<C: Component> std::fmt::Debug for Context<C>
where
    C::Props: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("props", &self.props)
            .field("depth", &self.depth)
            .finish_non_exhaustive()
    }
}

impl<C: Component> Context<C> {
    pub fn new(props: C::Props, link: Link<C>, contexts: ContextMap) -> Self {
        Self {
            props,
            link,
            contexts,
            depth: 0,
            _marker: PhantomData,
        }
    }

    pub fn get_context<T: Clone + 'static>(&self) -> Option<T> {
        self.contexts.get::<T>()
    }
}

pub trait Component: Sized + 'static {
    type Props: Props;

    type Msg: Debug + 'static;

    fn create(ctx: &Context<Self>) -> Result<Self, ComponentError>;

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Msg) -> Result<Cmd<Self>, ComponentError>;

    fn view(&self, ctx: &Context<Self>) -> Result<VNode, RenderError>;

    fn mounted(&mut self, _ctx: &Context<Self>) -> Result<(), ComponentError> {
        Ok(())
    }

    fn before_update(&mut self, _ctx: &Context<Self>) -> Result<(), ComponentError> {
        Ok(())
    }

    fn updated(&mut self, _ctx: &Context<Self>) -> Result<(), ComponentError> {
        Ok(())
    }

    fn before_unmount(&mut self, _ctx: &Context<Self>) -> Result<(), ComponentError> {
        Ok(())
    }

    fn should_render(&self, old_props: &Self::Props, new_props: &Self::Props) -> bool {
        old_props != new_props
    }

    fn on_error(&self, _ctx: &Context<Self>, _err: &ComponentError) -> Option<VNode> {
        None
    }

    fn name() -> &'static str {
        std::any::type_name::<Self>()
    }

    fn validate_props(props: &Self::Props) -> Result<(), PropsError> {
        props.validate()
    }
}

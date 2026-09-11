use crate::error::{ComponentError, RenderError};
use std::panic::{catch_unwind, AssertUnwindSafe};

fn payload_to_string(payload: &Box<dyn std::any::Any + Send + 'static>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".to_owned()
    }
}

pub fn run_guarded<C: crate::Component, T>(
    component: &str,
    stage: &str,
    location: &std::panic::Location<'_>,
    f: impl FnOnce() -> Result<T, ComponentError>,
) -> Result<T, ComponentError> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(r) => r,
        Err(payload) => Err(ComponentError::Panicked {
            component: component.to_owned(),
            stage: stage.to_owned(),
            location: location.to_string(),
            payload: payload_to_string(&payload),
        }),
    }
}

pub fn run_render_guarded<T>(
    location: &std::panic::Location<'_>,
    f: impl FnOnce() -> Result<T, RenderError>,
) -> Result<T, RenderError> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(r) => r,
        Err(payload) => Err(RenderError::Panicked {
            location: location.to_string(),
            payload: payload_to_string(&payload),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, Props, VNode};

    struct Dummy;
    #[derive(Clone, PartialEq, Debug)]
    struct DummyProps;
    impl Props for DummyProps {}
    impl crate::Component for Dummy {
        type Props = DummyProps;
        type Msg = ();
        fn create(_ctx: &Context<Self>) -> Result<Self, ComponentError> {
            Ok(Dummy)
        }
        fn update(
            &mut self,
            _ctx: &Context<Self>,
            _msg: (),
        ) -> Result<crate::Cmd<Self>, ComponentError> {
            Ok(crate::Cmd::None)
        }
        fn view(&self, _ctx: &Context<Self>) -> Result<VNode, crate::RenderError> {
            Ok(VNode::Empty)
        }
    }

    #[test]
    fn panic_becomes_error_not_unwind() {
        let loc = std::panic::Location::caller();
        let r: Result<(), ComponentError> =
            run_guarded::<Dummy, ()>("Test", "view", loc, || panic!("boom"));
        assert!(matches!(r, Err(ComponentError::Panicked { .. })));
    }
}

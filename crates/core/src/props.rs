use crate::error::PropsError;
use std::fmt::Debug;

pub trait Props: Clone + PartialEq + Debug {

    fn validate(&self) -> Result<(), PropsError> {
        Ok(())
    }

    fn component_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl Props for () {}

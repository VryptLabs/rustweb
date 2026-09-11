use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RenderError {

    #[error("render panicked at {location}: {payload}")]
    Panicked {

        location: String,

        payload: String,
    },

    #[error("invalid virtual DOM: {0}")]
    InvalidTree(String),

    #[error("rejected raw html: {0}")]
    RejectedRawHtml(String),

    #[error("render failed in {component}: {message}")]
    Other {

        component: String,

        message: String,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PropsError {

    #[error("missing required prop `{prop}` on component `{component}`")]
    MissingRequired {

        component: String,

        prop: String,
    },

    #[error("invalid prop `{prop}` on `{component}`: {hint}")]
    Invalid {

        component: String,

        prop: String,

        hint: String,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ComponentError {

    #[error("create failed for {component}: {message}")]
    Create {

        component: String,

        message: String,
    },

    #[error("update failed for {component} on msg {msg}: {message}")]
    Update {

        component: String,

        msg: String,

        message: String,
    },

    #[error("lifecycle `{hook}` failed for {component}: {message}")]
    Lifecycle {

        component: String,

        hook: String,

        message: String,
    },

    #[error("component {component} panicked in {stage} at {location}: {payload}")]
    Panicked {

        component: String,

        stage: String,

        location: String,

        payload: String,
    },

    #[error(transparent)]
    Render(#[from] RenderError),

    #[error(transparent)]
    Props(#[from] PropsError),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum HydrationError {

    #[error("hydration tag mismatch at {hid}: server had <{server}>, client wants <{client}>")]
    TagMismatch {

        hid: String,

        server: String,

        client: String,
    },

    #[error("hydration text mismatch at {hid}: server {server:?} vs client {client:?}")]
    TextMismatch {

        hid: String,

        server: String,

        client: String,
    },

    #[error("hydration structure mismatch at {hid}: {message}")]
    Structure {

        hid: String,

        message: String,
    },

    #[error("hydration checksum mismatch: server {server} vs client {client}")]
    Checksum {

        server: String,

        client: String,
    },
}

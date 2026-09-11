use rustweb_core::{AttrValue, VNode};

pub type PatchPath = Vec<usize>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Patch {
    Replace {
        path: PatchPath,

        new: VNode,
    },

    Create {
        path: PatchPath,

        index: usize,

        node: VNode,
    },

    Remove {
        path: PatchPath,
    },

    SetText {
        path: PatchPath,

        text: String,
    },

    SetAttr {
        path: PatchPath,

        name: String,

        value: AttrValue,
    },

    RemoveAttr {
        path: PatchPath,

        name: String,
    },

    Move {
        path: PatchPath,

        from: usize,

        to: usize,

        key: String,
    },

    SetListener {
        path: PatchPath,

        event: String,

        handler_id: String,
    },

    RemoveListener {
        path: PatchPath,

        event: String,
    },
}

impl Patch {
    pub fn is_move(&self) -> bool {
        matches!(self, Patch::Move { .. })
    }

    pub fn path(&self) -> &PatchPath {
        match self {
            Patch::Replace { path, .. }
            | Patch::Remove { path }
            | Patch::SetText { path, .. }
            | Patch::SetAttr { path, .. }
            | Patch::RemoveAttr { path, .. }
            | Patch::Create { path, .. }
            | Patch::Move { path, .. }
            | Patch::SetListener { path, .. }
            | Patch::RemoveListener { path, .. } => path,
        }
    }
}

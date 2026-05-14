/// Defines an import (`use` statement).
///
/// Represents `use path::ty;` with an optional visibility modifier.
#[derive(Debug, Clone)]
pub struct Import {
    /// Module path, e.g. `"std::collections"`.
    pub path: String,

    /// Imported type name, e.g. `"HashMap"`.
    pub ty: String,

    /// Optional visibility, e.g. `"pub"`.
    pub vis: Option<String>,
}

impl Import {
    /// Return a new import for `use path::ty;`.
    pub fn new(path: impl ToString, ty: impl ToString) -> Self {
        Import {
            path: path.to_string(),
            ty: ty.to_string(),
            vis: None,
        }
    }

    /// Set the import visibility.
    pub fn vis(&mut self, vis: impl ToString) -> &mut Self {
        self.vis = Some(vis.to_string());
        self
    }
}

use std::fmt::{self, Write};

/// Defines a type.
#[derive(Debug, Clone)]
pub struct Type {
    pub(crate) name: String,
    generics: Vec<Type>,
}

impl Type {
    /// Return a new type with the given name.
    pub fn new(name: impl ToString) -> Self {
        Type {
            name: name.to_string(),
            generics: Vec::new(),
        }
    }

    /// Add a generic to the type.
    pub fn generic<T>(&mut self, ty: T) -> &mut Self
    where
        T: Into<Type>,
    {
        assert!(
            !self.name.contains("<"),
            "type name already includes generics"
        );

        self.generics.push(ty.into());
        self
    }

    /// Rewrite the `Type` with the provided path prefix.
    ///
    /// # Panics
    ///
    /// Panics if the type name already contains a path separator (`::`)
    /// since prepending a path to a qualified name is ambiguous.
    pub fn path(&self, path: impl ToString) -> Type {
        assert!(
            !self.name.contains("::"),
            "type name already contains a path separator"
        );

        let mut name = path.to_string();
        name.push_str("::");
        name.push_str(&self.name);

        Type {
            name,
            generics: self.generics.clone(),
        }
    }

    /// Formats the struct using the given formatter.
    pub fn fmt(&self, dst: &mut String) -> fmt::Result {
        write!(dst, "{}", self.name)?;
        Type::fmt_slice(&self.generics, dst)
    }

    fn fmt_slice(generics: &[Type], dst: &mut String) -> fmt::Result {
        if !generics.is_empty() {
            write!(dst, "<")?;

            for (i, ty) in generics.iter().enumerate() {
                if i != 0 {
                    write!(dst, ", ")?
                }
                ty.fmt(dst)?;
            }

            write!(dst, ">")?;
        }

        Ok(())
    }
}

impl<S: ToString> From<S> for Type {
    fn from(src: S) -> Self {
        Type {
            name: src.to_string(),
            generics: vec![],
        }
    }
}

impl<'a> From<&'a Type> for Type {
    fn from(src: &'a Type) -> Self {
        src.clone()
    }
}

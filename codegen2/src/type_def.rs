use std::fmt::{self, Write};

use crate::bound::Bound;
use crate::docs::Docs;
use crate::formatter::fmt_bounds;
use crate::r#type::Type;

/// Defines a type definition.
#[derive(Debug, Clone)]
pub struct TypeDef {
    pub ty: Type,
    vis: Option<String>,
    docs: Option<Docs>,
    derive: Vec<String>,
    allow: Vec<String>,
    attributes: Vec<String>,
    repr: Option<String>,
    bounds: Vec<Bound>,
    macros: Vec<String>,
}

impl TypeDef {
    /// Return a structure definition with the provided name
    pub fn new(name: impl ToString) -> Self {
        TypeDef {
            ty: Type::new(name),
            vis: None,
            docs: None,
            derive: Vec::new(),
            allow: Vec::new(),
            attributes: Vec::new(),
            repr: None,
            bounds: Vec::new(),
            macros: Vec::new(),
        }
    }

    pub fn vis(&mut self, vis: impl ToString) {
        self.vis = Some(vis.to_string());
    }

    pub fn bound<T>(&mut self, name: impl ToString, ty: T)
    where
        T: Into<Type>,
    {
        self.bounds.push(Bound {
            name: name.to_string(),
            bound: vec![ty.into()],
        });
    }

    pub fn r#macro(&mut self, r#macro: impl ToString) {
        self.macros.push(r#macro.to_string());
    }

    pub fn attr(&mut self, attr: impl ToString) {
        self.attributes.push(attr.to_string());
    }

    pub fn doc(&mut self, docs: impl ToString) {
        self.docs = Some(Docs::new(docs));
    }

    pub fn derive(&mut self, name: impl ToString) {
        self.derive.push(name.to_string());
    }

    pub fn allow(&mut self, allow: impl ToString) {
        self.allow.push(allow.to_string());
    }

    pub fn repr(&mut self, repr: impl ToString) {
        self.repr = Some(repr.to_string());
    }

    pub fn fmt_head(
        &self,
        keyword: &str,
        parents: &[Type],
        dst: &mut String,
    ) -> fmt::Result {
        if let Some(ref docs) = self.docs {
            docs.fmt(dst)?;
        }

        self.fmt_allow(dst)?;
        self.fmt_derive(dst)?;
        self.fmt_repr(dst)?;
        self.fmt_attributes(dst)?;
        self.fmt_macros(dst)?;

        if let Some(ref vis) = self.vis {
            write!(dst, "{} ", vis)?;
        }

        write!(dst, "{} ", keyword)?;
        self.ty.fmt(dst)?;

        if !parents.is_empty() {
            for (i, ty) in parents.iter().enumerate() {
                if i == 0 {
                    write!(dst, ": ")?;
                } else {
                    write!(dst, " + ")?;
                }

                ty.fmt(dst)?;
            }
        }

        fmt_bounds(&self.bounds, dst)?;

        Ok(())
    }

    fn fmt_attributes(&self, dst: &mut String) -> fmt::Result {
        for attr in &self.attributes {
            writeln!(dst, "#[{}]", attr)?;
        }

        Ok(())
    }

    fn fmt_allow(&self, dst: &mut String) -> fmt::Result {
        for allow in &self.allow {
            writeln!(dst, "#[allow({})]", allow)?;
        }

        Ok(())
    }

    fn fmt_repr(&self, dst: &mut String) -> fmt::Result {
        if let Some(ref repr) = self.repr {
            writeln!(dst, "#[repr({})]", repr)?;
        }

        Ok(())
    }

    fn fmt_derive(&self, dst: &mut String) -> fmt::Result {
        if !self.derive.is_empty() {
            write!(dst, "#[derive(")?;

            for (i, name) in self.derive.iter().enumerate() {
                if i != 0 {
                    write!(dst, ", ")?
                }
                write!(dst, "{}", name)?;
            }

            writeln!(dst, ")]")?;
        }

        Ok(())
    }

    fn fmt_macros(&self, dst: &mut String) -> fmt::Result {
        for m in self.macros.iter() {
            writeln!(dst, "{}", m)?;
        }
        Ok(())
    }
}

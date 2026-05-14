use std::fmt::{self, Write};

use crate::associated_const::AssociatedConst;
use crate::associated_type::AssociatedType;
use crate::bound::Bound;
use crate::function::Function;
use crate::r#type::Type;
use crate::type_def::TypeDef;
use crate::util::{fmt_bound_rhs, write_block};

/// Define a trait.
#[derive(Debug, Clone)]
pub struct Trait {
    type_def: TypeDef,
    parents: Vec<Type>,
    associated_consts: Vec<AssociatedConst>,
    attributes: Vec<String>,
    associated_tys: Vec<AssociatedType>,
    fns: Vec<Function>,
}

impl Trait {
    /// Return a trait definition with the provided name
    pub fn new(name: impl ToString) -> Self {
        Trait {
            type_def: TypeDef::new(name),
            parents: Vec::new(),
            associated_consts: Vec::new(),
            attributes: Vec::new(),
            associated_tys: Vec::new(),
            fns: Vec::new(),
        }
    }

    /// Returns a reference to the type
    pub fn ty(&self) -> &Type {
        &self.type_def.ty
    }

    /// Set the trait visibility.
    pub fn vis(&mut self, vis: impl ToString) -> &mut Self {
        self.type_def.vis(vis);
        self
    }

    /// Attr
    pub fn attr(&mut self, attr: impl ToString) -> &mut Self {
        self.attributes.push(attr.to_string());
        self
    }

    /// Add a generic to the trait
    pub fn generic(&mut self, name: impl ToString) -> &mut Self {
        self.type_def.ty.generic(name);
        self
    }

    /// Add a `where` bound to the trait.
    pub fn bound<T>(&mut self, name: impl ToString, ty: T) -> &mut Self
    where
        T: Into<Type>,
    {
        self.type_def.bound(name, ty);
        self
    }

    /// Add a macro to the trait def (e.g. `"#[async_trait]"`)
    pub fn r#macro(&mut self, r#macro: impl ToString) -> &mut Self {
        self.type_def.r#macro(r#macro);
        self
    }

    /// Add a parent trait.
    pub fn parent<T>(&mut self, ty: T) -> &mut Self
    where
        T: Into<Type>,
    {
        self.parents.push(ty.into());
        self
    }

    /// Set the trait documentation.
    pub fn doc(&mut self, docs: impl ToString) -> &mut Self {
        self.type_def.doc(docs);
        self
    }

    /// Add an associated const. Returns a mutable reference to the new
    /// associated const for futher configuration.
    pub fn associated_const<T>(&mut self, name: impl ToString, ty: T) -> &mut AssociatedConst
    where
        T: Into<Type>,
    {
        self.associated_consts.push(AssociatedConst(Bound {
            name: name.to_string(),
            bound: vec![ty.into()],
        }));

        self.associated_consts
            .last_mut()
            .expect("associated_consts was just pushed to")
    }

    /// Add an associated type. Returns a mutable reference to the new
    /// associated type for futher configuration.
    pub fn associated_type(&mut self, name: impl ToString) -> &mut AssociatedType {
        self.associated_tys.push(AssociatedType(Bound {
            name: name.to_string(),
            bound: vec![],
        }));

        self.associated_tys
            .last_mut()
            .expect("associated_tys was just pushed to")
    }

    /// Push a new function definition, returning a mutable reference to it.
    pub fn new_fn(&mut self, name: impl ToString) -> &mut Function {
        let mut func = Function::new(name);
        func.body = None;

        self.push_fn(func);
        self.fns.last_mut().expect("fns was just pushed to")
    }

    /// Push a function definition.
    pub fn push_fn(&mut self, item: Function) -> &mut Self {
        self.fns.push(item);
        self
    }

    /// Formats the scope using the given formatter.
    pub fn fmt(&self, dst: &mut String) -> fmt::Result {
        for attr in &self.attributes {
            writeln!(dst, "#[{}]", attr)?;
        }

        self.type_def.fmt_head("trait", &self.parents, dst)?;

        write_block(dst, |dst| {
            let assoc_csts = &self.associated_consts;
            let assoc_tys = &self.associated_tys;

            // format associated types
            if !assoc_csts.is_empty() {
                for cst in assoc_csts {
                    let cst = &cst.0;

                    write!(dst, "const {}", cst.name)?;

                    if !cst.bound.is_empty() {
                        write!(dst, ": ")?;
                        fmt_bound_rhs(&cst.bound, dst)?;
                    }

                    writeln!(dst, ";")?;
                }
            }

            // format associated types
            if !assoc_tys.is_empty() {
                for ty in assoc_tys {
                    let ty = &ty.0;

                    write!(dst, "type {}", ty.name)?;

                    if !ty.bound.is_empty() {
                        write!(dst, ": ")?;
                        fmt_bound_rhs(&ty.bound, dst)?;
                    }

                    writeln!(dst, ";")?;
                }
            }

            for (i, func) in self.fns.iter().enumerate() {
                if i != 0 || !assoc_tys.is_empty() || !assoc_csts.is_empty() {
                    writeln!(dst)?;
                }

                func.fmt(true, dst)?;
            }

            Ok(())
        })
    }
}

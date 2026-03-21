use std::fmt::{self, Display, Write};
use std::process::Command;

use indexmap::IndexMap;

use crate::docs::Docs;
use crate::r#enum::Enum;
use crate::formatter::Formatter;
use crate::function::Function;
use crate::r#impl::Impl;
use crate::import::Import;
use crate::item::Item;
use crate::module::Module;
use crate::r#struct::Struct;
use crate::r#trait::Trait;
use crate::type_alias::TypeAlias;

/// Defines a scope.
///
/// A scope contains modules, types, etc...
#[derive(Debug, Clone)]
pub struct Scope {
    /// Scope documentation
    docs: Option<Docs>,

    /// Imports
    imports: IndexMap<String, IndexMap<String, Import>>,

    /// Contents of the documentation,
    items: Vec<Item>,
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    /// Returns a new scope
    pub fn new() -> Self {
        Scope {
            docs: None,
            imports: IndexMap::new(),
            items: vec![],
        }
    }

    /// Set the scope documentation.
    pub fn doc(&mut self, docs: impl ToString) -> &mut Self {
        self.docs = Some(Docs::new(docs));
        self
    }

    /// Import a type into the scope.
    ///
    /// This results in a new `use` statement being added to the beginning of
    /// the scope.
    pub fn import(&mut self, path: impl ToString, ty: impl ToString) -> &mut Import {
        // Only import the root segment when given a qualified path like "a::B".
        let ty = ty.to_string();
        let ty = ty.split("::").next().unwrap_or(ty.as_str());
        self.imports
            .entry(path.to_string())
            .or_default()
            .entry(ty.to_string())
            .or_insert_with(|| Import::new(path, ty))
    }

    /// Push a new module definition, returning a mutable reference to it.
    ///
    /// # Panics
    ///
    /// Since a module's name must uniquely identify it within the scope in
    /// which it is defined, pushing a module whose name is already defined
    /// in this scope will cause this function to panic.
    ///
    /// In many cases, the [`get_or_new_module`] function is preferrable, as it
    /// will return the existing definition instead.
    ///
    /// [`get_or_new_module`]: #method.get_or_new_module
    pub fn new_module(&mut self, name: impl ToString) -> &mut Module {
        self.push_module(Module::new(name));
        let Some(Item::Module(v)) = self.items.last_mut() else {
            unreachable!()
        };
        v
    }

    /// Returns a mutable reference to a module if it is exists in this scope.
    pub fn get_module_mut<Q: ?Sized>(&mut self, name: &Q) -> Option<&mut Module>
    where
        String: PartialEq<Q>,
    {
        self.items
            .iter_mut()
            .filter_map(|item| match item {
                &mut Item::Module(ref mut module) if module.name == *name => Some(module),
                _ => None,
            })
            .next()
    }

    /// Returns a mutable reference to a module if it is exists in this scope.
    pub fn get_module<Q: ?Sized>(&self, name: &Q) -> Option<&Module>
    where
        String: PartialEq<Q>,
    {
        self.items
            .iter()
            .filter_map(|item| match item {
                Item::Module(module) if module.name == *name => Some(module),
                _ => None,
            })
            .next()
    }

    /// Returns a mutable reference to a module, creating it if it does
    /// not exist.
    pub fn get_or_new_module<Q: ?Sized + Display>(&mut self, name: &Q) -> &mut Module
    where
        String: PartialEq<Q>,
    {
        if self.get_module(name).is_some() {
            self.get_module_mut(name)
                .expect("module existence was just checked")
        } else {
            self.new_module(name)
        }
    }

    /// Push a [`Module`] definition.
    ///
    /// # Panics
    ///
    /// Since a module's name must uniquely identify it within the scope in
    /// which it is defined, pushing a module whose name is already defined
    /// in this scope will cause this function to panic.
    ///
    /// In many cases, the [`get_or_new_module`] function is preferable, as it will
    /// return the existing definition instead.
    ///
    /// [`get_or_new_module`]: #method.get_or_new_module
    pub fn push_module(&mut self, item: Module) -> &mut Self {
        assert!(self.get_module(&item.name).is_none());
        self.items.push(Item::Module(item));
        self
    }

    /// Push a new [`Struct`] definition, returning a mutable reference to it.
    pub fn new_struct(&mut self, name: impl ToString) -> &mut Struct {
        self.push_struct(Struct::new(name));
        let Some(Item::Struct(v)) = self.items.last_mut() else {
            unreachable!()
        };
        v
    }

    /// Push a [`Struct`] definition
    pub fn push_struct(&mut self, item: Struct) -> &mut Self {
        self.items.push(Item::Struct(item));
        self
    }

    /// Iterate over named child modules stored directly in this scope.
    ///
    /// Returned item tuple is `(module_name, module_ref)`.
    pub fn modules(&self) -> impl Iterator<Item = (&str, &Module)> + '_ {
        self.items.iter().filter_map(|item| match item {
            Item::Module(m) => Some((m.name.as_str(), m)),
            _ => None,
        })
    }

    /// Returns a mutable reference to a [`Struct`] if it is existing in this scope.
    pub fn get_struct_mut<Q: ?Sized>(&mut self, name: &Q) -> Option<&mut Struct>
    where
        String: PartialEq<Q>,
    {
        self.items.iter_mut().find_map(|item| match item {
            Item::Struct(stru) if stru.ty().name == *name => Some(stru),
            _ => None,
        })
    }

    /// Push a new function definition, returning a mutable reference to it.
    pub fn new_fn(&mut self, name: impl ToString) -> &mut Function {
        self.push_fn(Function::new(name));
        let Some(Item::Function(v)) = self.items.last_mut() else {
            unreachable!()
        };
        v
    }

    /// Push a function definition
    pub fn push_fn(&mut self, item: Function) -> &mut Self {
        self.items.push(Item::Function(item));
        self
    }

    /// Returns a mutable reference to a [`Function`] if it is existing in this scope.
    pub fn get_fn_mut<Q: ?Sized>(&mut self, name: &Q) -> Option<&mut Function>
    where
        String: PartialEq<Q>,
    {
        self.items.iter_mut().find_map(|item| match item {
            Item::Function(fun) if fun.name == *name => Some(fun),
            _ => None,
        })
    }

    /// Push a new trait definition, returning a mutable reference to it.
    pub fn new_trait(&mut self, name: impl ToString) -> &mut Trait {
        self.push_trait(Trait::new(name));
        let Some(Item::Trait(v)) = self.items.last_mut() else {
            unreachable!()
        };
        v
    }

    /// Push a trait definition
    pub fn push_trait(&mut self, item: Trait) -> &mut Self {
        self.items.push(Item::Trait(item));
        self
    }

    /// Push a new struct definition, returning a mutable reference to it.
    pub fn new_enum(&mut self, name: impl ToString) -> &mut Enum {
        self.push_enum(Enum::new(name));
        let Some(Item::Enum(v)) = self.items.last_mut() else {
            unreachable!()
        };
        v
    }

    /// Push a structure definition
    pub fn push_enum(&mut self, item: Enum) -> &mut Self {
        self.items.push(Item::Enum(item));
        self
    }

    /// Returns a mutable reference to an enum if it is existing in this scope.
    pub fn get_enum_mut<Q: ?Sized>(&mut self, name: &Q) -> Option<&mut Enum>
    where
        String: PartialEq<Q>,
    {
        self.items.iter_mut().find_map(|item| match item {
            Item::Enum(enu) if enu.ty().name == *name => Some(enu),
            _ => None,
        })
    }

    /// Push a new `impl` block, returning a mutable reference to it.
    pub fn new_impl(&mut self, target: impl ToString) -> &mut Impl {
        self.push_impl(Impl::new(target));
        let Some(Item::Impl(v)) = self.items.last_mut() else {
            unreachable!()
        };
        v
    }

    /// Push an `impl` block.
    pub fn push_impl(&mut self, item: Impl) -> &mut Self {
        self.items.push(Item::Impl(item));
        self
    }

    /// Push a raw string to the scope.
    ///
    /// This string will be included verbatim in the formatted string.
    pub fn raw(&mut self, val: impl ToString) -> &mut Self {
        self.items.push(Item::Raw(val.to_string()));
        self
    }

    /// Push a new `TypeAlias`, returning a mutable reference to it.
    pub fn new_type_alias(&mut self, name: impl ToString, target: impl ToString) -> &mut TypeAlias {
        self.push_type_alias(TypeAlias::new(name, target));
        let Some(Item::TypeAlias(v)) = self.items.last_mut() else {
            unreachable!()
        };
        v
    }

    /// Push an `TypeAlias`.
    pub fn push_type_alias(&mut self, item: TypeAlias) -> &mut Self {
        self.items.push(Item::TypeAlias(item));
        self
    }

    /// Return a `rustfmt`-formatted string representation of the scope.
    ///
    /// Falls back to the built-in formatter when `rustfmt` is not available.
    #[expect(
        clippy::inherent_to_string,
        reason = "return type differs from Display convention"
    )]
    pub fn to_string(&self) -> String {
        let mut ret = String::new();

        self.fmt(&mut Formatter::new(&mut ret))
            .expect("formatting to String cannot fail");

        if let Some(b'\n') = ret.as_bytes().last() {
            ret.pop();
        }

        rustfmt(&ret).unwrap_or(ret)
    }

    /// Formats the scope using the given formatter.
    pub fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        if let Some(ref docs) = self.docs {
            docs.fmt(fmt)?;
        }

        self.fmt_imports(fmt)?;

        if !self.imports.is_empty() {
            writeln!(fmt)?;
        }

        for (i, item) in self.items.iter().enumerate() {
            if i != 0 {
                writeln!(fmt)?;
            }

            match *item {
                Item::Module(ref v) => v.fmt(fmt)?,
                Item::Struct(ref v) => v.fmt(fmt)?,
                Item::Function(ref v) => v.fmt(false, fmt)?,
                Item::Trait(ref v) => v.fmt(fmt)?,
                Item::Enum(ref v) => v.fmt(fmt)?,
                Item::Impl(ref v) => v.fmt(fmt)?,
                Item::Raw(ref v) => {
                    writeln!(fmt, "{}", v)?;
                }
                Item::TypeAlias(ref v) => v.fmt(fmt)?,
            }
        }

        Ok(())
    }

    fn fmt_imports(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        // Emit one `use` per import; rustfmt merges and sorts them.
        for imports in self.imports.values() {
            for import in imports.values() {
                if let Some(ref vis) = import.vis {
                    write!(fmt, "{} ", vis)?;
                }
                writeln!(fmt, "use {}::{};", import.path, import.ty)?;
            }
        }
        Ok(())
    }
}

/// Run nightly `rustfmt` on a source string, returning `None` if the
/// toolchain is unavailable or formatting fails.
fn rustfmt(source: &str) -> Option<String> {
    let mut child = Command::new("rustup")
        .args(["run", "nightly", "rustfmt"])
        .arg("--edition")
        .arg("2024")
        .arg("--config")
        .arg("imports_granularity=Module,group_imports=StdExternalCrate")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;

    use std::io::Write as _;
    child.stdin.take()?.write_all(source.as_bytes()).ok()?;

    let output = child.wait_with_output().ok()?;
    if !output.status.success() {
        return None;
    }

    let mut formatted = String::from_utf8(output.stdout).ok()?;
    // Match the convention: no trailing newline
    if formatted.ends_with('\n') {
        formatted.pop();
    }
    Some(formatted)
}

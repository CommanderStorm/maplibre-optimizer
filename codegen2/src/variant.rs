use std::fmt::{self, Write};

use crate::Field;
use crate::fields::Fields;
use crate::formatter::Formatter;
use crate::r#type::Type;

/// Defines an enum variant.
#[derive(Debug, Clone)]
pub struct Variant {
    name: String,
    fields: Fields,
    /// Documentation for the variant.
    documentation: String,
    /// Annotations for field e.g., `#[serde(rename = "variant")]`.
    annotations: Vec<String>,
    discriminant: Option<String>,
}

impl Variant {
    /// Return a new enum variant with the given name.
    pub fn new(name: impl ToString) -> Self {
        Variant {
            name: name.to_string(),
            fields: Fields::Empty,
            documentation: String::new(),
            annotations: Vec::new(),
            discriminant: None,
        }
    }

    /// Add a named field to the variant.
    ///
    /// An enum variant can either set named fields with this function or tuple fields
    /// with [`tuple`](Self::tuple), but not both.
    pub fn push_named(&mut self, field: Field) -> &mut Self {
        self.fields.push_named(field);
        self
    }

    /// Add a named field to the variant.
    ///
    /// An enum variant can either set named fields with this function or tuple fields
    /// with [`tuple`](Self::tuple), but not both.
    pub fn named<T>(&mut self, name: impl ToString, ty: T) -> &mut Self
    where
        T: Into<Type>,
    {
        self.fields.named(name, ty);
        self
    }

    /// Create a named field for the enum.
    ///
    /// An enum variant can either set named fields with this function or tuple fields
    /// with [`tuple`](Self::tuple), but not both.
    pub fn new_named<T>(&mut self, name: impl ToString, ty: T) -> &mut Field
    where
        T: Into<Type>,
    {
        self.fields.new_named(name, ty)
    }

    /// Add a tuple field to the variant.
    ///
    /// An enum variant can either be a tuple with this function or have named fields
    /// with [`named`](Self::named), but not both.
    pub fn tuple(&mut self, ty: impl ToString) -> &mut Self {
        self.fields.tuple(ty);
        self
    }

    /// Set the variant documentation.
    pub fn doc(&mut self, documentation: impl ToString) -> &mut Self {
        self.documentation = documentation.to_string();
        self
    }

    /// Add an anotation to the variant.
    pub fn annotation(&mut self, annotation: impl Into<String>) -> &mut Self {
        self.annotations.push(annotation.into());
        self
    }

    /// Set the discriminant value for the variant.
    pub fn discriminant(&mut self, discriminant: impl ToString) -> &mut Self {
        self.discriminant = Some(discriminant.to_string());
        self
    }

    /// Formats the variant using the given formatter.
    pub fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        if !self.documentation.is_empty() {
            for doc in self.documentation.lines() {
                writeln!(fmt, "/// {}", doc)?;
            }
        }
        for a in &self.annotations {
            write!(fmt, "{}", a)?;
            writeln!(fmt)?;
        }
        write!(fmt, "{}", self.name)?;
        self.fields.fmt(fmt)?;
        if let Some(ref discriminant) = self.discriminant {
            write!(fmt, " = {}", discriminant)?;
        }
        writeln!(fmt, ",")?;

        Ok(())
    }
}

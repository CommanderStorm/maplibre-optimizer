use std::fmt::{self, Write};

use crate::field::Field;
use crate::formatter::Formatter;
use crate::r#type::Type;

/// One slot in a tuple struct or tuple enum variant, optionally with outer attributes.
#[derive(Debug, Clone)]
pub struct TupleField {
    /// Lines like `#[serde(rename = "x")]` (without surrounding `#[]` wrapper per line — use full attribute text).
    pub annotations: Vec<String>,
    /// Field type.
    pub ty: Type,
}

impl TupleField {
    /// Tuple slot with no attributes.
    pub fn new<T>(ty: T) -> Self
    where
        T: Into<Type>,
    {
        Self {
            annotations: Vec::new(),
            ty: ty.into(),
        }
    }

    /// Tuple slot with attribute lines (each string is a full attribute, e.g. `#[cfg_attr(...)]`).
    pub fn with_annotations<I, S, T>(annotations: I, ty: T) -> Self
    where
        I: IntoIterator<Item = S>,
        S: ToString,
        T: Into<Type>,
    {
        Self {
            annotations: annotations.into_iter().map(|s| s.to_string()).collect(),
            ty: ty.into(),
        }
    }
}

/// Defines a set of fields.
#[derive(Debug, Clone)]
pub enum Fields {
    Empty,
    Tuple(Vec<TupleField>),
    Named(Vec<Field>),
}

impl Fields {
    pub fn push_named(&mut self, field: Field) -> &mut Self {
        match *self {
            Fields::Empty => {
                *self = Fields::Named(vec![field]);
            }
            Fields::Named(ref mut fields) => {
                fields.push(field);
            }
            _ => panic!("field list is named"),
        }

        self
    }

    pub fn named<T>(&mut self, name: impl ToString, ty: T) -> &mut Self
    where
        T: Into<Type>,
    {
        self.push_named(Field {
            name: name.to_string(),
            ty: ty.into(),
            documentation: String::new(),
            annotation: Vec::new(),
            value: String::new(),
            visibility: None,
        })
    }

    pub fn new_named<T>(&mut self, name: impl ToString, ty: T) -> &mut Field
    where
        T: Into<Type>,
    {
        self.named(name, ty);
        if let Fields::Named(ref mut fields) = *self {
            fields.last_mut().expect("fields was just pushed to")
        } else {
            unreachable!()
        }
    }

    /// Append a tuple slot with no attributes.
    pub fn tuple<T>(&mut self, ty: T) -> &mut Self
    where
        T: Into<Type>,
    {
        self.tuple_field(TupleField::new(ty))
    }

    /// Append a tuple slot, optionally with attribute lines on that slot.
    pub fn tuple_field(&mut self, slot: TupleField) -> &mut Self {
        match *self {
            Fields::Empty => {
                *self = Fields::Tuple(vec![slot]);
            }
            Fields::Tuple(ref mut fields) => {
                fields.push(slot);
            }
            _ => panic!("field list is tuple"),
        }

        self
    }

    /// Append a tuple slot with attributes before the type.
    pub fn tuple_with_attrs<I, S, T>(&mut self, annotations: I, ty: T) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: ToString,
        T: Into<Type>,
    {
        self.tuple_field(TupleField::with_annotations(annotations, ty))
    }

    pub fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Fields::Named(ref fields) => {
                assert!(!fields.is_empty());

                fmt.block(|fmt| {
                    for f in fields {
                        if !f.documentation.is_empty() {
                            for doc in f.documentation.lines() {
                                writeln!(fmt, "/// {}", doc)?;
                            }
                        }
                        if !f.annotation.is_empty() {
                            for ann in &f.annotation {
                                writeln!(fmt, "{}", ann)?;
                            }
                        }
                        if let Some(ref visibility) = f.visibility {
                            write!(fmt, "{} ", visibility)?;
                        }
                        write!(fmt, "{}: ", f.name)?;
                        f.ty.fmt(fmt)?;
                        writeln!(fmt, ",")?;
                    }

                    Ok(())
                })?;
            }
            Fields::Tuple(ref slots) => {
                assert!(!slots.is_empty());

                // Emit all slots inline; rustfmt handles line-breaking.
                write!(fmt, "(")?;
                for (i, slot) in slots.iter().enumerate() {
                    if i > 0 {
                        write!(fmt, ", ")?;
                    }
                    for ann in &slot.annotations {
                        write!(fmt, "{} ", ann)?;
                    }
                    slot.ty.fmt(fmt)?;
                }
                write!(fmt, ")")?;
            }
            Fields::Empty => {}
        }

        Ok(())
    }
}

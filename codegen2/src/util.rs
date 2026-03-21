use std::fmt::{self, Write};

use crate::bound::Bound;
use crate::r#type::Type;

/// Emit `{ <body> }\n`, where `<body>` is produced by the closure.
///
/// Indentation is left to `rustfmt`.
pub fn write_block<F>(dst: &mut String, f: F) -> fmt::Result
where
    F: FnOnce(&mut String) -> fmt::Result,
{
    writeln!(dst, " {{")?;
    f(dst)?;
    writeln!(dst, "}}")?;
    Ok(())
}

/// Format generics.
pub fn fmt_generics(generics: &[String], dst: &mut String) -> fmt::Result {
    if !generics.is_empty() {
        write!(dst, "<")?;

        for (i, ty) in generics.iter().enumerate() {
            if i != 0 {
                write!(dst, ", ")?
            }
            write!(dst, "{}", ty)?;
        }

        write!(dst, ">")?;
    }

    Ok(())
}

/// Format generic bounds.
pub fn fmt_bounds(bounds: &[Bound], dst: &mut String) -> fmt::Result {
    if !bounds.is_empty() {
        // Emit a flat `where` clause; rustfmt handles alignment.
        write!(dst, " where ")?;
        for (i, bound) in bounds.iter().enumerate() {
            if i != 0 {
                write!(dst, ", ")?;
            }
            write!(dst, "{}: ", bound.name)?;
            fmt_bound_rhs(&bound.bound, dst)?;
        }
    }
    Ok(())
}

/// Format multiple generic bounds.
pub fn fmt_bound_rhs(tys: &[Type], dst: &mut String) -> fmt::Result {
    for (i, ty) in tys.iter().enumerate() {
        if i != 0 {
            write!(dst, " + ")?
        }
        ty.fmt(dst)?;
    }

    Ok(())
}

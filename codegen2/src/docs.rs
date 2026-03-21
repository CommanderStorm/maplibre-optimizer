use std::fmt::{self, Write};

#[derive(Debug, Clone)]
pub struct Docs {
    pub(crate) docs: String,
}

impl Docs {
    pub fn new(docs: impl ToString) -> Self {
        Docs {
            docs: docs.to_string(),
        }
    }

    pub fn fmt(&self, dst: &mut String) -> fmt::Result {
        for line in self.docs.lines() {
            write!(dst, "///")?;
            if !line.is_empty() {
                write!(dst, " {}", line)?;
            }
            writeln!(dst)?;
        }

        Ok(())
    }
}

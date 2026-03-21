use std::fmt::{self, Write};

use crate::block::Block;

#[derive(Debug, Clone)]
pub enum Body {
    String(String),
    Block(Block),
}

impl Body {
    pub fn fmt(&self, dst: &mut String) -> fmt::Result {
        match &self {
            Body::String(s) => writeln!(dst, "{}", s),
            Body::Block(b) => b.fmt(dst),
        }
    }
}

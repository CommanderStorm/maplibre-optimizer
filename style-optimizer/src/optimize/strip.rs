//! Strip metadata from typed style structures.

use maplibre_style_spec::spec::{AnyLayer, MaplibreStyleSpecification};

/// Remove `metadata` from the root and all layers.
pub(crate) fn strip_metadata(style: &mut MaplibreStyleSpecification) {
    style.metadata = None;

    for layer in &mut style.layers {
        match layer {
            AnyLayer::Typed(t) => t.common_mut().metadata = None,
            AnyLayer::Ref(r) => r.metadata = None,
        }
    }
}

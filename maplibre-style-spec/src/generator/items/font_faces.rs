use codegen2::Scope;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields) {
    let font_with_range = scope
        .new_struct("FontWithRange")
        .vis("pub")
        .doc("Font file URL and the unicode-range at which it can be used")
        .derive("serde::Deserialize, PartialEq, Eq, Debug, Clone");
    font_with_range
        .new_field("url", "url::Url")
        .vis("pub")
        .doc("URL the font can retrieved under");
    font_with_range
        .new_field("unicode_range", "String")
        .vis("pub")
        .doc("Unicode characters where this font should be used")
        .annotation("#[serde(rename=\"unicode-range\")]");

    let enu = scope
        .new_enum("FontFace")
        .vis("pub")
        .attr("serde(untagged)")
        .derive("serde::Deserialize, PartialEq, Eq, Debug, Clone");
    enu.new_variant("Url")
        .doc("A single global font file URL")
        .tuple("url::Url");
    enu.new_variant("FontRange")
        .doc("Load different fonts depending on the unicode range")
        .tuple("Vec<FontWithRange>");

    scope
        .new_struct(name)
        .vis("pub")
        .doc(&common.doc)
        .derive("serde::Deserialize, PartialEq, Eq, Debug, Clone")
        .tuple_field("std::collections::BTreeMap<String,FontFace>");

    generate_test_from_example_if_present(scope, name, common.example.as_ref());
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::decoder::StyleReference;
    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(&mut scope, "Foo", &Fields::default());
        insta::assert_snapshot!(scope.to_string(), @r#"
        /// Font file URL and the unicode-range at which it can be used
        #[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone)]
        pub struct FontWithRange {
            /// URL the font can retrieved under
            pub url: url::Url,
            /// Unicode characters where this font should be used
            #[serde(rename="unicode-range")]
            pub unicode_range: String,
        }

        #[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone)]
        #[serde(untagged)]
        pub enum FontFace {
            /// A single global font file URL
            Url(url::Url),
            /// Load different fonts depending on the unicode range
            FontRange(Vec<FontWithRange>),
        }

        #[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone)]
        pub struct Foo(std::collections::BTreeMap<String,FontFace>);
        "#)
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "font-faces": {
          "type": "fontFaces",
          "doc": "The `font-faces` property can be used to specify what font files to use for rendering text. Font faces contain information needed to render complex texts such as [Devanagari](https://en.wikipedia.org/wiki/Devanagari), [Khmer](https://en.wikipedia.org/wiki/Khmer_script) among many others.<h2>Unicode range</h2>The optional `unicode-range` property can be used to only use a particular font file for characters within the specified unicode range(s). Its value should be an array of strings, each indicating a start and end of a unicode range, similar to the [CSS descriptor with the same name](https://developer.mozilla.org/en-US/docs/Web/CSS/@font-face/unicode-range). This allows specifying multiple non-consecutive unicode ranges. When not specified, the default value is `U+0-10FFFF`, meaning the font file will be used for all unicode characters.\n\nRefer to the [Unicode Character Code Charts](https://www.unicode.org/charts/) to see ranges for scripts supported by Unicode. To see what unicode code-points are available in a font, use a tool like [FontDrop](https://fontdrop.info/).\n\n<h2>Font Resolution</h2>For every name in a symbol layer’s [`text-font`](./layers.md/#text-font) array, characters are matched if they are covered one of the by the font files in the corresponding entry of the `font-faces` map. Any still-unmatched characters then fall back to the [`glyphs`](./glyphs.md) URL if provided.\n\n<h2>Supported Fonts</h2>What type of fonts are supported is implementation-defined. Unsupported fonts are ignored.",
          "example": {
            "Noto Sans Regular": [{
              "url": "https://cdn.jsdelivr.net/gh/notofonts/notofonts.github.io/fonts/NotoSansKhmer/hinted/ttf/NotoSansKhmer-Regular.ttf",
              "unicode-range": ["U+1780-17FF"]
            },
              {
                "url": "https://cdn.jsdelivr.net/gh/notofonts/notofonts.github.io/fonts/NotoSansDevanagari/hinted/ttf/NotoSansDevanagari-Regular.ttf",
                "unicode-range": ["U+0900-097F"]
              },
              {
                "url": "https://cdn.jsdelivr.net/gh/notofonts/notofonts.github.io/fonts/NotoSansMyanmar/hinted/ttf/NotoSansMyanmar-Regular.ttf",
                "unicode-range": ["U+1000-109F"]
              },
              {
                "url": "https://cdn.jsdelivr.net/gh/notofonts/notofonts.github.io/fonts/NotoSansEthiopic/hinted/ttf/NotoSansEthiopic-Regular.ttf",
                "unicode-range": ["U+1200-137F"]
              }],
            "Unifont": "https://ftp.gnu.org/gnu/unifont/unifont-15.0.01/unifont-15.0.01.ttf"
          },
          "sdk-support": {
            "basic functionality": {
              "js": "https://github.com/maplibre/maplibre-gl-js/issues/6637",
              "android": "11.13.0",
              "ios": "6.18.0"
            }
          }
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        /// Font file URL and the unicode-range at which it can be used
        #[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone)]
        pub struct FontWithRange {
            /// URL the font can retrieved under
            pub url: url::Url,
            /// Unicode characters where this font should be used
            #[serde(rename="unicode-range")]
            pub unicode_range: String,
        }

        #[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone)]
        #[serde(untagged)]
        pub enum FontFace {
            /// A single global font file URL
            Url(url::Url),
            /// Load different fonts depending on the unicode range
            FontRange(Vec<FontWithRange>),
        }

        /// The `font-faces` property can be used to specify what font files to use for rendering text. Font faces contain information needed to render complex texts such as [Devanagari](https://en.wikipedia.org/wiki/Devanagari), [Khmer](https://en.wikipedia.org/wiki/Khmer_script) among many others.<h2>Unicode range</h2>The optional `unicode-range` property can be used to only use a particular font file for characters within the specified unicode range(s). Its value should be an array of strings, each indicating a start and end of a unicode range, similar to the [CSS descriptor with the same name](https://developer.mozilla.org/en-US/docs/Web/CSS/@font-face/unicode-range). This allows specifying multiple non-consecutive unicode ranges. When not specified, the default value is `U+0-10FFFF`, meaning the font file will be used for all unicode characters.
        ///
        /// Refer to the [Unicode Character Code Charts](https://www.unicode.org/charts/) to see ranges for scripts supported by Unicode. To see what unicode code-points are available in a font, use a tool like [FontDrop](https://fontdrop.info/).
        ///
        /// <h2>Font Resolution</h2>For every name in a symbol layer’s [`text-font`](./layers.md/#text-font) array, characters are matched if they are covered one of the by the font files in the corresponding entry of the `font-faces` map. Any still-unmatched characters then fall back to the [`glyphs`](./glyphs.md) URL if provided.
        ///
        /// <h2>Supported Fonts</h2>What type of fonts are supported is implementation-defined. Unsupported fonts are ignored.
        #[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone)]
        pub struct FontFaces(std::collections::BTreeMap<String,FontFace>);

        #[cfg(test)] 
        mod test {
            use super::*;

            #[test]
            fn test_example_font_faces_decodes() {
                let example = serde_json::json!({"Noto Sans Regular":[{"unicode-range":["U+1780-17FF"],"url":"https://cdn.jsdelivr.net/gh/notofonts/notofonts.github.io/fonts/NotoSansKhmer/hinted/ttf/NotoSansKhmer-Regular.ttf"},{"unicode-range":["U+0900-097F"],"url":"https://cdn.jsdelivr.net/gh/notofonts/notofonts.github.io/fonts/NotoSansDevanagari/hinted/ttf/NotoSansDevanagari-Regular.ttf"},{"unicode-range":["U+1000-109F"],"url":"https://cdn.jsdelivr.net/gh/notofonts/notofonts.github.io/fonts/NotoSansMyanmar/hinted/ttf/NotoSansMyanmar-Regular.ttf"},{"unicode-range":["U+1200-137F"],"url":"https://cdn.jsdelivr.net/gh/notofonts/notofonts.github.io/fonts/NotoSansEthiopic/hinted/ttf/NotoSansEthiopic-Regular.ttf"}],"Unifont":"https://ftp.gnu.org/gnu/unifont/unifont-15.0.01/unifont-15.0.01.ttf"});
                let _ = serde_json::from_value::<FontFaces>(example).expect("example should decode");
            }
        }
        "#);
    }
}

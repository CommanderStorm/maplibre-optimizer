use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::fuzz;
use crate::mir::types::FieldMeta;

const SPRITE_URL_AND_ID_DOCS: &str = r#"Defining Id and Url for a sprite allows you to load multiple sprites at once.

When given the following sprite defintion, the sprite loader will load the sprites from the given urls.

```json
[
{
"id": "roadsigns",
"url": "https://example.com/myroadsigns"
},
{
"id": "shops",
"url": "https://example2.com/someurl"
},
{
"id": "default",
"url": "https://example2.com/anotherurl"
}
]
```

As you can see, each sprite has an id.
All images contained within a sprite also have an id.
When using multiple sprites, you need to prefix the id of the image with the id of the sprite it is contained within, followed by a colon.
For example, to reference the `stop_sign` image on the `roadsigns` sprite, you would need to use `roadsigns:stop_sign`.

The sprite with id `default` is special in that you do not need to prefix the images contained within it.
For example, to reference the image with id `airport` in the default sprite above, you can simply use `airport`.
"#;

pub fn generate(scope: &mut Scope, name: &str, meta: &FieldMeta) {
    let sprite_url_and_id = scope
        .new_struct("SpriteUrlAndId")
        .vis("pub")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY)
        .doc(SPRITE_URL_AND_ID_DOCS);
    sprite_url_and_id
        .field("id", "std::string::String")
        .doc("Identifier of a sprite");
    sprite_url_and_id
        .new_field("url", "url::Url")
        .vis("pub")
        .annotation(fuzz::ARB_URL)
        .doc(
            r#"URL where the sprite can be loaded from.

This is equivalent to the following multiple sprite definition:

```json
{
        "id": "default",
        "url": "https://example2.com/anotherurl"
}
```"#,
        );

    let enu = scope
        .new_enum(name)
        .vis("pub")
        .doc(&meta.doc)
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY);

    enu.new_variant("Url")
        .doc("URL where the sprite can be loaded from")
        .tuple_with_attrs([fuzz::ARB_URL], "url::Url");
    enu.new_variant("Multiple")
        .doc("Array of `{ id: ..., url: ... }` pairs to load multiple sprites")
        .tuple("Vec<SpriteUrlAndId>");

    generate_test_from_example_if_present(scope, name, meta.example.as_ref());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(&mut scope, "Foo", &FieldMeta::default());
        insta::assert_snapshot!(scope.to_string(), @r#"
        /// Identifier of a sprite
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct SpriteUrlAndId {
            id: std::string::String,
            /// URL where the sprite can be loaded from.
            ///
            /// This is equivalent to the following multiple sprite definition:
            ///
            /// ```json
            /// {
            ///         "id": "default",
            ///         "url": "https://example2.com/anotherurl"
            /// }
            /// ```
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_url))]
            pub url: url::Url,
        }

        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub enum Foo {
            /// URL where the sprite can be loaded from
            Url(
                #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_url))]
                url::Url,
            ),
            /// Array of `{ id: ..., url: ... }` pairs to load multiple sprites
            Multiple(Vec<SpriteUrlAndId>),
        }
        "#)
    }
}

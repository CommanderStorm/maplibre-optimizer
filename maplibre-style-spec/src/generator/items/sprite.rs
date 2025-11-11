use codegen::Scope;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields) {
    let sprite_url_and_id = scope.new_struct("SpriteUrlAndId")
        .vis("pub")
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .doc(r#"Defining Id and Url for a sprite allows you to load multiple sprites at once.

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
"#);
    sprite_url_and_id
        .field("id", "String")
        .doc("Identifier of a sprite");
    sprite_url_and_id.field("url", "url::Url").doc(
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
        .doc(&common.doc)
        .derive("serde::Deserialize, PartialEq, Debug, Clone");

    enu.new_variant("Url")
        .doc("URL where the sprite can be loaded from")
        .tuple("url::Url");
    enu.new_variant("Multiple")
        .doc("Array of `{ id: ..., url: ... }` pairs to load multiple sprites")
        .tuple("Vec<SpriteUrlAndId>");

    generate_test_from_example_if_present(scope, name, common);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(&mut scope, "Foo", &Fields::default());
        insta::assert_snapshot!(scope.to_string(), @r#"
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
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct SpriteUrlAndId {
            id: String,
            url: url::Url,
        }

        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub enum Foo {
            /// URL where the sprite can be loaded from
            Url(url::Url),
            /// Array of `{ id: ..., url: ... }` pairs to load multiple sprites
            Multiple(Vec<SpriteUrlAndId>),
        }
        "#)
    }
}

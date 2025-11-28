use codegen2::Scope;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field("Sources");
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
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        struct Foo(Sources);
        ")
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "sources": {
          "required": true,
          "type": "sources",
          "doc": "Sources state which data the map should display. Specify the type of source with the `type` property. Adding a source isn't enough to make data appear on the map because sources don't contain styling details like color or width. Layers refer to a source and give it a visual representation. This makes it possible to style the same source in different ways, like differentiating between types of roads in a highways layer.\n\nTiled sources (vector and raster) must specify their details according to the [TileJSON specification](https://github.com/mapbox/tilejson-spec).",
          "example": {
            "maplibre-demotiles": {
              "type": "vector",
              "url": "https://demotiles.maplibre.org/tiles/tiles.json"
            },
            "maplibre-tilejson": {
              "type": "vector",
              "url": "http://api.example.com/tilejson.json"
            },
            "maplibre-streets": {
              "type": "vector",
              "tiles": [
                  "http://a.example.com/tiles/{z}/{x}/{y}.pbf",
                  "http://b.example.com/tiles/{z}/{x}/{y}.pbf"
              ],
              "maxzoom": 14
            },
            "wms-imagery": {
              "type": "raster",
              "tiles": [
                  "http://a.example.com/wms?bbox={bbox-epsg-3857}&format=image/png&service=WMS&version=1.1.1&request=GetMap&srs=EPSG:3857&width=256&height=256&layers=example"
              ],
              "tileSize": 256
            }
          }
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        /// Sources state which data the map should display. Specify the type of source with the `type` property. Adding a source isn't enough to make data appear on the map because sources don't contain styling details like color or width. Layers refer to a source and give it a visual representation. This makes it possible to style the same source in different ways, like differentiating between types of roads in a highways layer.
        ///
        /// Tiled sources (vector and raster) must specify their details according to the [TileJSON specification](https://github.com/mapbox/tilejson-spec).
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        struct Sources(Sources);

        #[cfg(test)]
        mod test {
            use super::*;

            #[test]
            fn test_example_sources_decodes() {
                let example = serde_json::json!({"maplibre-demotiles":{"type":"vector","url":"https://demotiles.maplibre.org/tiles/tiles.json"},"maplibre-streets":{"maxzoom":14,"tiles":["http://a.example.com/tiles/{z}/{x}/{y}.pbf","http://b.example.com/tiles/{z}/{x}/{y}.pbf"],"type":"vector"},"maplibre-tilejson":{"type":"vector","url":"http://api.example.com/tilejson.json"},"wms-imagery":{"tileSize":256,"tiles":["http://a.example.com/wms?bbox={bbox-epsg-3857}&format=image/png&service=WMS&version=1.1.1&request=GetMap&srs=EPSG:3857&width=256&height=256&layers=example"],"type":"raster"}});
                let _ = serde_json::from_value::<Sources>(example).expect("example should decode");
            }
        }
        "#);
    }
}

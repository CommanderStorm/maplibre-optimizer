use codegen2::*;

#[test]
fn empty_scope() {
    let scope = Scope::new();

    assert_eq!(scope.to_string(), "");
}

#[test]
fn single_struct() {
    let mut scope = Scope::new();

    scope
        .new_struct("Foo")
        .field("one", "usize")
        .field("two", "String");

    insta::assert_snapshot!(scope.to_string(), @r"
    struct Foo {
        one: usize,
        two: String,
    }
    ");
}
#[test]
fn type_alias() {
    let mut scope = Scope::new();

    scope.new_type_alias("hello", "world").vis("pub");

    insta::assert_snapshot!(scope.to_string(), @"pub type hello = world;");
}

#[test]
fn struct_with_pushed_field() {
    let mut scope = Scope::new();
    let mut struct_ = Struct::new("Foo");
    let field = Field::new("one", "usize");
    struct_.push_field(field);
    scope.push_struct(struct_);

    insta::assert_snapshot!(scope.to_string(), @r"
    struct Foo {
        one: usize,
    }
    ");
}

// Carter 8/13/22 I found this test broken because both annotation and documentation do not support vectors
// I'm reducing this test down to single line versions, but unclear how this test appeared broken
#[test]
fn single_struct_documented_field() {
    let mut scope = Scope::new();

    let doc = "Field's documentation\nSecond line";

    let mut struct_ = Struct::new("Foo");

    let mut field1 = Field::new("one", "usize");
    field1.doc(doc);
    struct_.push_field(field1);

    let mut field2 = Field::new("two", "usize");
    field2.annotation("#[serde(rename = \"bar\")]");
    struct_.push_field(field2);

    let mut field3 = Field::new("three", "usize");
    field3
        .doc(doc)
        .annotation("#[serde(skip_serializing)]\n#[serde(skip_deserializing)]");
    struct_.push_field(field3);

    scope.push_struct(struct_);

    insta::assert_snapshot!(scope.to_string(), @r#"
    struct Foo {
        /// Field's documentation
        /// Second line
        one: usize,
        #[serde(rename = "bar")]
        two: usize,
        /// Field's documentation
        /// Second line
        #[serde(skip_serializing)]
        #[serde(skip_deserializing)]
        three: usize,
    }
    "#);
}

#[test]
fn single_fn() {
    let mut scope = Scope::new();
    scope
        .new_fn("my_fn")
        .vis("pub")
        .arg("foo", Type::new("uint"))
        .ret(Type::new("uint"))
        .line("let res = foo + 1;")
        .line("res");

    insta::assert_snapshot!(scope.to_string(), @r"
    pub fn my_fn(foo: uint) -> uint {
        let res = foo + 1;
        res
    }
    ");
}

#[test]
fn empty_struct() {
    let mut scope = Scope::new();

    scope.new_struct("Foo");

    insta::assert_snapshot!(scope.to_string(), @"struct Foo;");
}

#[test]
fn two_structs() {
    let mut scope = Scope::new();

    scope
        .new_struct("Foo")
        .field("one", "usize")
        .field("two", "String");

    scope.new_struct("Bar").field("hello", "World");

    insta::assert_snapshot!(scope.to_string(), @r"
    struct Foo {
        one: usize,
        two: String,
    }

    struct Bar {
        hello: World,
    }
    ");
}

#[test]
fn struct_with_derive() {
    let mut scope = Scope::new();

    scope
        .new_struct("Foo")
        .derive("Debug")
        .derive("Clone")
        .field("one", "usize")
        .field("two", "String");

    insta::assert_snapshot!(scope.to_string(), @r"
    #[derive(Debug, Clone)]
    struct Foo {
        one: usize,
        two: String,
    }
    ");
}

#[test]
fn struct_with_repr() {
    let mut scope = Scope::new();

    scope
        .new_struct("Foo")
        .repr("C")
        .field("one", "u8")
        .field("two", "u8");

    insta::assert_snapshot!(scope.to_string(), @r"
    #[repr(C)]
    struct Foo {
        one: u8,
        two: u8,
    }
    ");
}

#[test]
fn struct_with_allow() {
    let mut scope = Scope::new();

    scope
        .new_struct("Foo")
        .allow("dead_code")
        .field("one", "u8")
        .field("two", "u8");

    insta::assert_snapshot!(scope.to_string(), @r"
    #[allow(dead_code)]
    struct Foo {
        one: u8,
        two: u8,
    }
    ");
}

#[test]
fn struct_with_generics_1() {
    let mut scope = Scope::new();

    scope
        .new_struct("Foo")
        .generic("T")
        .generic("U")
        .field("one", "T")
        .field("two", "U");

    insta::assert_snapshot!(scope.to_string(), @r"
    struct Foo<T, U> {
        one: T,
        two: U,
    }
    ");
}

#[test]
fn struct_with_generics_2() {
    let mut scope = Scope::new();

    scope
        .new_struct("Foo")
        .generic("T, U")
        .field("one", "T")
        .field("two", "U");

    insta::assert_snapshot!(scope.to_string(), @r"
    struct Foo<T, U> {
        one: T,
        two: U,
    }
    ");
}

#[test]
fn struct_with_generics_3() {
    let mut scope = Scope::new();

    scope
        .new_struct("Foo")
        .generic("T: Win, U")
        .field("one", "T")
        .field("two", "U");

    insta::assert_snapshot!(scope.to_string(), @r"
    struct Foo<T: Win, U> {
        one: T,
        two: U,
    }
    ");
}

#[test]
fn struct_where_clause_1() {
    let mut scope = Scope::new();

    scope
        .new_struct("Foo")
        .generic("T")
        .bound("T", "Foo")
        .field("one", "T");

    insta::assert_snapshot!(scope.to_string(), @r"
    struct Foo<T>
    where T: Foo,
    {
        one: T,
    }
    ");
}

#[test]
fn struct_where_clause_2() {
    let mut scope = Scope::new();

    scope
        .new_struct("Foo")
        .generic("T, U")
        .bound("T", "Foo")
        .bound("U", "Baz")
        .field("one", "T")
        .field("two", "U");

    insta::assert_snapshot!(scope.to_string(), @r"
    struct Foo<T, U>
    where T: Foo,
          U: Baz,
    {
        one: T,
        two: U,
    }
    ");
}

#[test]
fn struct_doc() {
    let mut scope = Scope::new();

    scope
        .new_struct("Foo")
        .doc(
            "Hello, this is a doc string\n\
              that continues on another line.",
        )
        .field("one", "T");

    insta::assert_snapshot!(scope.to_string(), @r"
    /// Hello, this is a doc string
    /// that continues on another line.
    struct Foo {
        one: T,
    }
    ");
}

#[test]
fn function_doc() {
    let mut scope = Scope::new();

    let f = scope.new_fn("pet_toby");
    f.line("println!(\"petting Toby many times because he is such a good boi\");");
    f.doc("This is a function comment.");

    insta::assert_snapshot!(scope.to_string(), @r#"
    /// This is a function comment.
    fn pet_toby() {
        println!("petting Toby many times because he is such a good boi");
    }
    "#);
}

#[test]
fn scope_doc() {
    let mut scope = Scope::new();
    scope.doc("This is a scope comment.\nThis is a newline");

    insta::assert_snapshot!(scope.to_string(), @r"
    /// This is a scope comment.
    /// This is a newline
    ");
}

#[test]
fn module_doc() {
    let mut scope = Scope::new();
    let m = scope.new_module("toby");
    m.doc("This is a module comment.");

    insta::assert_snapshot!(scope.to_string(), @r"
    /// This is a module comment.
    mod toby {
    }
    ");
}

#[test]
fn struct_in_mod() {
    let mut scope = Scope::new();

    {
        let module = scope.new_module("foo");
        module
            .new_struct("Foo")
            .doc("Hello some docs")
            .derive("Debug")
            .generic("T, U")
            .bound("T", "SomeBound")
            .bound("U", "SomeOtherBound")
            .field("one", "T")
            .field("two", "U");
    }

    insta::assert_snapshot!(scope.to_string(), @r"
    mod foo {
        /// Hello some docs
        #[derive(Debug)]
        struct Foo<T, U>
        where T: SomeBound,
              U: SomeOtherBound,
        {
            one: T,
            two: U,
        }
    }
    ");
}

#[test]
fn struct_mod_import() {
    let mut scope = Scope::new();
    scope
        .new_module("foo")
        .import("bar", "Bar")
        .new_struct("Foo")
        .field("bar", "Bar");

    insta::assert_snapshot!(scope.to_string(), @r"
    mod foo {
        use bar::Bar;

        struct Foo {
            bar: Bar,
        }
    }
    ");
}
#[test]
fn type_alias_in_mod() {
    let mut scope = Scope::new();

    {
        let module = scope.new_module("foo");
        module.new_type_alias("hello", "world").vis("pub");
    }

    insta::assert_snapshot!(scope.to_string(), @r"
    mod foo {
        pub type hello = world;
    }
    ");
}

#[test]
fn enum_with_repr() {
    let mut scope = Scope::new();

    scope
        .new_enum("IpAddrKind")
        .repr("u8")
        .push_variant(Variant::new("V4"))
        .push_variant(Variant::new("V6"));

    insta::assert_snapshot!(scope.to_string(), @r"
    #[repr(u8)]
    enum IpAddrKind {
        V4,
        V6,
    }
    ");
}

#[test]
fn enum_variant_with_doc() {
    let mut scope = Scope::new();

    let enu = scope.new_enum("Foo");
    enu.new_variant("Bar")
        .doc("Documentation for Bar variant")
        .new_named("bo", "bool")
        .doc("Documentation for the named bo field");
    enu.new_variant("Baz")
        .doc("Documentation for Baz variant\nWith multiple lines");
    enu.new_variant("Bazinga")
        .doc("Documentation for Bazinga variant")
        .tuple("String");

    insta::assert_snapshot!(scope.to_string(), @r"
    enum Foo {
        /// Documentation for Bar variant
        Bar {
            /// Documentation for the named bo field
            bo: bool,
        }
        ,
        /// Documentation for Baz variant
        /// With multiple lines
        Baz,
        /// Documentation for Bazinga variant
        Bazinga(String),
    }
    ");
}

#[test]
fn enum_with_discriminants() {
    let mut scope = Scope::new();

    let enu = scope.new_enum("IpAddrKind").repr("u8");
    enu.new_variant("V4").discriminant("4");
    enu.new_variant("V6").discriminant("6");

    insta::assert_snapshot!(scope.to_string(), @r"
    #[repr(u8)]
    enum IpAddrKind {
        V4 = 4,
        V6 = 6,
    }
    ");
}

#[test]
fn enum_with_allow() {
    let mut scope = Scope::new();

    scope
        .new_enum("IpAddrKind")
        .allow("dead_code")
        .push_variant(Variant::new("V4"))
        .push_variant(Variant::new("V6"));

    insta::assert_snapshot!(scope.to_string(), @r"
    #[allow(dead_code)]
    enum IpAddrKind {
        V4,
        V6,
    }
    ");
}

#[test]
fn scoped_imports() {
    let mut scope = Scope::new();
    scope
        .new_module("foo")
        .import("bar", "Bar")
        .import("bar", "baz::Baz")
        .import("bar::quux", "quuux::Quuuux")
        .new_struct("Foo")
        .field("bar", "Bar")
        .field("baz", "baz::Baz")
        .field("quuuux", "quuux::Quuuux");

    insta::assert_snapshot!(scope.to_string(), @r"
    mod foo {
        use bar::{Bar, baz};
        use bar::quux::quuux;

        struct Foo {
            bar: Bar,
            baz: baz::Baz,
            quuuux: quuux::Quuuux,
        }
    }
    ");
}

#[test]
fn module_mut() {
    let mut scope = Scope::new();
    scope.new_module("foo").import("bar", "Bar");

    scope
        .get_module_mut("foo")
        .expect("module_mut")
        .new_struct("Foo")
        .field("bar", "Bar");

    insta::assert_snapshot!(scope.to_string(), @r"
    mod foo {
        use bar::Bar;

        struct Foo {
            bar: Bar,
        }
    }
    ");
}

#[test]
fn get_or_new_module() {
    let mut scope = Scope::new();
    assert!(scope.get_module("foo").is_none());

    scope.get_or_new_module("foo").import("bar", "Bar");

    scope
        .get_or_new_module("foo")
        .new_struct("Foo")
        .field("bar", "Bar");

    insta::assert_snapshot!(scope.to_string(), @r"
    mod foo {
        use bar::Bar;

        struct Foo {
            bar: Bar,
        }
    }
    ");
}

#[test]
fn function_with_async() {
    let mut scope = Scope::new();
    let trt = scope.new_trait("Foo");

    let f = trt.new_fn("pet_toby");
    f.set_async(true);
    f.line("println!(\"petting toby because he is a good boi\");");

    insta::assert_snapshot!(scope.to_string(), @r#"
    trait Foo {
        async fn pet_toby() {
            println!("petting toby because he is a good boi");
        }
    }
    "#);
}

#[test]
fn function_with_const() {
    let mut scope = Scope::new();

    scope
        .new_fn("one_plus_one")
        .set_const(true)
        .ret("u32")
        .line("1 + 1");

    insta::assert_snapshot!(scope.to_string(), @r"
    const fn one_plus_one() -> u32 {
        1 + 1
    }
    ");
}

#[test]
fn trait_with_macros() {
    let mut scope = Scope::new();
    let trt = scope.new_trait("Foo");
    trt.r#macro("#[async_trait]");
    trt.r#macro("#[toby_is_cute]");

    let f = trt.new_fn("pet_toby");
    f.set_async(true);
    f.line("println!(\"petting toby because he is a good boi\");");

    insta::assert_snapshot!(scope.to_string(), @r##"
    #[async_trait]
    #[toby_is_cute]
    trait Foo {
        async fn pet_toby() {
            println!("petting toby because he is a good boi");
        }
    }
    "##);
}

#[test]
fn impl_with_macros() {
    let mut scope = Scope::new();
    scope.new_struct("Bar");
    let imp = scope.new_impl("Bar");
    imp.impl_trait("Foo");
    imp.r#macro("#[async_trait]");
    imp.r#macro("#[toby_is_cute]");

    let f = imp.new_fn("pet_toby");
    f.set_async(true);
    f.line("println!(\"petting Toby many times because he is such a good boi\");");

    insta::assert_snapshot!(scope.to_string(), @r#"
    struct Bar;

    #[async_trait]
    #[toby_is_cute]
    impl Foo for Bar {
        async fn pet_toby() {
            println!("petting Toby many times because he is such a good boi");
        }
    }
    "#);
}

#[test]
fn struct_with_multiple_allow() {
    let mut scope = Scope::new();

    scope
        .new_struct("Foo")
        .allow("dead_code")
        .allow("clippy::all")
        .field("one", "u8")
        .field("two", "u8");

    insta::assert_snapshot!(scope.to_string(), @r"
    #[allow(dead_code)]
    #[allow(clippy::all)]
    struct Foo {
        one: u8,
        two: u8,
    }
    ");
}

#[test]
fn enum_with_multiple_allow() {
    let mut scope = Scope::new();

    scope
        .new_enum("IpAddrKind")
        .allow("dead_code")
        .allow("clippy::all")
        .push_variant(Variant::new("V4"))
        .push_variant(Variant::new("V6"));

    insta::assert_snapshot!(scope.to_string(), @r"
    #[allow(dead_code)]
    #[allow(clippy::all)]
    enum IpAddrKind {
        V4,
        V6,
    }
    ");
}

#[test]
fn impl_with_associated_const() {
    let mut scope = Scope::new();

    let bar = scope.new_trait("Bar");
    bar.associated_const("CONST_NAME", Type::new("f32"));

    let _foo = scope.new_struct("Foo");

    let foo_impl = scope.new_impl("Foo");
    foo_impl.impl_trait("Bar");
    foo_impl.associate_const("CONST_NAME", Type::new("f32"), "0.0", "pub");

    insta::assert_snapshot!(scope.to_string(), @r"
    trait Bar {
        const CONST_NAME: f32;
    }

    struct Foo;

    impl Bar for Foo {
        pub const CONST_NAME: f32 = 0.0;
    }
    ");
}

#[test]
fn struct_with_member_visibility() {
    let mut scope = Scope::new();

    let struct_description = scope.new_struct("Foo");

    let mut bar = Field::new("bar", "usize");
    bar.vis("pub");

    struct_description.push_field(bar);
    struct_description.new_field("baz", "i16").vis("pub(crate)");

    insta::assert_snapshot!(scope.to_string(), @r"
        struct Foo {
            pub bar: usize,
            pub(crate) baz: i16,
        }
        ");
}

#[test]
fn get_mut_struct() {
    let mut scope = Scope::new();
    assert!(scope.get_enum_mut("Foo").is_none());

    scope.new_struct("Foo").field("one", "usize");
    insta::assert_snapshot!(scope.to_string(), @r"
    struct Foo {
        one: usize,
    }
    ");

    scope
        .get_struct_mut("Foo")
        .expect("struct_mut")
        .field("two", "String");
    insta::assert_snapshot!(scope.to_string(), @r"
    struct Foo {
        one: usize,
        two: String,
    }
    ");
}
#[test]
fn get_mut_enum() {
    let mut scope = Scope::new();
    assert!(scope.get_enum_mut("Bar").is_none());

    scope.new_enum("Bar").push_variant(Variant::new("Baz"));
    insta::assert_snapshot!(scope.to_string(), @r"
    enum Bar {
        Baz,
    }
    ");

    scope
        .get_enum_mut("Bar")
        .expect("enum_mut")
        .push_variant(Variant::new("Quux"));
    insta::assert_snapshot!(scope.to_string(), @r"
    enum Bar {
        Baz,
        Quux,
    }
    ");
}
#[test]
fn get_mut_fn() {
    let mut scope = Scope::new();
    assert!(scope.get_fn_mut("baz").is_none());

    scope.new_fn("baz").line("let a = 1;");
    insta::assert_snapshot!(scope.to_string(), @r"
    fn baz() {
        let a = 1;
    }
    ");

    scope.get_fn_mut("baz").expect("fn_mut").line("let b = 2;");
    insta::assert_snapshot!(scope.to_string(), @r"
    fn baz() {
        let a = 1;
        let b = 2;
    }
    ");
}

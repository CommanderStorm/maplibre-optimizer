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
fn tuple_struct_with_slot_attributes() {
    let mut scope = Scope::new();

    scope
        .new_struct("Lit")
        .derive("Debug")
        .tuple_field_with_attrs(["#[serde(transparent)]"], "i32");

    insta::assert_snapshot!(scope.to_string(), @r"
    #[derive(Debug)]
    struct Lit(#[serde(transparent)] i32);
    ");
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
    where
        T: Foo,
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
    where
        T: Foo,
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
    scope.doc("This is a scope comment.\n\nThis is a newline");

    insta::assert_snapshot!(scope.to_string(), @r"
    //! This is a scope comment.
    //!
    //! This is a newline
    ");
}

#[test]
fn module_doc() {
    let mut scope = Scope::new();
    let m = scope.new_module("toby");
    m.doc("This is a module comment.");

    insta::assert_snapshot!(scope.to_string(), @r"
    /// This is a module comment.
    mod toby {}
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
        where
            T: SomeBound,
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
        },
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
        use bar::quux::quuux;
        use bar::{Bar, baz};

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

    insta::assert_snapshot!(scope.to_string(), @r#"
    #[async_trait]
    #[toby_is_cute]
    trait Foo {
        async fn pet_toby() {
            println!("petting toby because he is a good boi");
        }
    }
    "#);
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

#[test]
fn type_with_generic() {
    let mut scope = Scope::new();
    let mut ty = Type::new("Vec");
    ty.generic("String");
    scope.new_fn("f").ret(ty).line("vec![]");

    insta::assert_snapshot!(scope.to_string(), @r#"
    fn f() -> Vec<String> {
        vec![]
    }
    "#);
}

#[test]
fn type_with_nested_generics() {
    let mut scope = Scope::new();
    let mut inner = Type::new("Option");
    inner.generic("T");
    let mut outer = Type::new("Vec");
    outer.generic(inner);
    scope.new_fn("f").ret(outer).line("vec![]");

    insta::assert_snapshot!(scope.to_string(), @r#"
    fn f() -> Vec<Option<T>> {
        vec![]
    }
    "#);
}

#[test]
fn type_path() {
    let ty = Type::new("HashMap");
    let qualified = ty.path("std::collections");
    let mut scope = Scope::new();
    scope
        .new_fn("f")
        .ret(qualified)
        .line("std::collections::HashMap::new()");

    insta::assert_snapshot!(scope.to_string(), @r"
    fn f() -> std::collections::HashMap {
        std::collections::HashMap::new()
    }
    ");
}

#[test]
fn type_path_preserves_generics() {
    let mut ty = Type::new("HashMap");
    ty.generic("K");
    ty.generic("V");
    let qualified = ty.path("std::collections");
    let mut scope = Scope::new();
    scope
        .new_fn("f")
        .ret(qualified)
        .line("std::collections::HashMap::new()");

    insta::assert_snapshot!(scope.to_string(), @r"
    fn f() -> std::collections::HashMap<K, V> {
        std::collections::HashMap::new()
    }
    ");
}

#[test]
#[should_panic(expected = "type name already contains a path separator")]
fn type_path_panics_on_qualified_name() {
    let ty = Type::new("std::HashMap");
    ty.path("collections");
}

#[test]
#[should_panic(expected = "type name already includes generics")]
fn type_generic_panics_when_name_contains_angle_brackets() {
    let mut ty = Type::new("Vec<String>");
    ty.generic("T");
}

#[test]
fn function_with_block() {
    let mut scope = Scope::new();
    let f = scope.new_fn("process");
    let mut blk = Block::new("for item in items");
    blk.line("println!(\"{}\", item);");
    f.push_block(blk);

    insta::assert_snapshot!(scope.to_string(), @r#"
    fn process() {
        for item in items {
            println!("{}", item);
        }
    }
    "#);
}

#[test]
fn nested_blocks() {
    let mut scope = Scope::new();
    let f = scope.new_fn("nested");
    let mut outer = Block::new("for i in 0..10");
    let mut inner = Block::new("if i > 5");
    inner.line("println!(\"{}\", i);");
    outer.push_block(inner);
    f.push_block(outer);

    insta::assert_snapshot!(scope.to_string(), @r#"
    fn nested() {
        for i in 0..10 {
            if i > 5 {
                println!("{}", i);
            }
        }
    }
    "#);
}

#[test]
fn block_with_after() {
    let mut scope = Scope::new();
    let f = scope.new_fn("with_else");
    let mut if_blk = Block::new("if condition");
    if_blk.line("a();");
    if_blk.after(" else { b(); }");
    f.push_block(if_blk);

    insta::assert_snapshot!(scope.to_string(), @r"
    fn with_else() {
        if condition {
            a();
        } else {
            b();
        }
    }
    ");
}


#[test]
fn function_with_ref_self() {
    let mut scope = Scope::new();
    let imp = scope.new_impl("Foo");
    imp.new_fn("bar")
        .arg_ref_self()
        .ret("usize")
        .line("self.x");

    insta::assert_snapshot!(scope.to_string(), @r"
    impl Foo {
        fn bar(&self) -> usize {
            self.x
        }
    }
    ");
}

#[test]
fn function_with_mut_self() {
    let mut scope = Scope::new();
    let imp = scope.new_impl("Foo");
    imp.new_fn("set").arg_mut_self().arg("val", "usize").line("self.x = val;");

    insta::assert_snapshot!(scope.to_string(), @r"
    impl Foo {
        fn set(&mut self, val: usize) {
            self.x = val;
        }
    }
    ");
}

#[test]
fn function_with_self() {
    let mut scope = Scope::new();
    let imp = scope.new_impl("Foo");
    imp.new_fn("into_inner").arg_self().ret("usize").line("self.x");

    insta::assert_snapshot!(scope.to_string(), @r"
    impl Foo {
        fn into_inner(self) -> usize {
            self.x
        }
    }
    ");
}

#[test]
fn function_with_generics_and_bounds() {
    let mut scope = Scope::new();
    scope
        .new_fn("serialize")
        .generic("T")
        .arg("value", "T")
        .ret("String")
        .bound("T", "Display")
        .line("value.to_string()");

    insta::assert_snapshot!(scope.to_string(), @r"
    fn serialize<T>(value: T) -> String
    where
        T: Display,
    {
        value.to_string()
    }
    ");
}

#[test]
fn function_with_allow() {
    let mut scope = Scope::new();
    scope
        .new_fn("unused")
        .allow("dead_code")
        .line("let _ = 1;");

    insta::assert_snapshot!(scope.to_string(), @r"
    #[allow(dead_code)]
    fn unused() {
        let _ = 1;
    }
    ");
}

#[test]
fn function_with_attr() {
    let mut scope = Scope::new();
    scope
        .new_fn("my_test")
        .attr("test")
        .line("assert!(true);");

    insta::assert_snapshot!(scope.to_string(), @r"
    #[test]
    fn my_test() {
        assert!(true);
    }
    ");
}

#[test]
fn function_with_extern_abi() {
    let mut scope = Scope::new();
    scope
        .new_fn("extern_fn")
        .extern_abi("C")
        .arg("x", "i32")
        .ret("i32")
        .line("x + 1");

    insta::assert_snapshot!(scope.to_string(), @r#"
    extern "C" fn extern_fn(x: i32) -> i32 {
        x + 1
    }
    "#);
}

#[test]
fn function_with_multiple_attrs() {
    let mut scope = Scope::new();
    scope
        .new_fn("handler")
        .attr("inline")
        .attr("must_use")
        .ret("bool")
        .line("true");

    insta::assert_snapshot!(scope.to_string(), @r"
    #[inline]
    #[must_use]
    fn handler() -> bool {
        true
    }
    ");
}

#[test]
fn trait_with_vis() {
    let mut scope = Scope::new();
    scope.new_trait("MyTrait").vis("pub");

    insta::assert_snapshot!(scope.to_string(), @"pub trait MyTrait {}");
}

#[test]
fn trait_with_doc() {
    let mut scope = Scope::new();
    scope
        .new_trait("MyTrait")
        .doc("A documented trait.\nWith two lines.");

    insta::assert_snapshot!(scope.to_string(), @r"
    /// A documented trait.
    /// With two lines.
    trait MyTrait {}
    ");
}

#[test]
fn trait_with_generics_and_bounds() {
    let mut scope = Scope::new();
    scope
        .new_trait("Convert")
        .generic("T")
        .bound("T", "Clone")
        .new_fn("convert")
        .arg("input", "T")
        .ret("Self");

    insta::assert_snapshot!(scope.to_string(), @r"
    trait Convert<T>
    where
        T: Clone,
    {
        fn convert(input: T) -> Self;
    }
    ");
}

#[test]
fn trait_with_parent() {
    let mut scope = Scope::new();
    scope
        .new_trait("ChildTrait")
        .parent("ParentTrait")
        .parent("Debug");

    insta::assert_snapshot!(scope.to_string(), @"trait ChildTrait: ParentTrait + Debug {}");
}

#[test]
fn trait_with_associated_type() {
    let mut scope = Scope::new();
    let trt = scope.new_trait("Iterator");
    trt.associated_type("Item");
    trt.new_fn("next")
        .arg_mut_self()
        .ret("Option<Self::Item>");

    insta::assert_snapshot!(scope.to_string(), @r"
    trait Iterator {
        type Item;

        fn next(&mut self) -> Option<Self::Item>;
    }
    ");
}

#[test]
fn trait_with_associated_type_bound() {
    let mut scope = Scope::new();
    let trt = scope.new_trait("Foo");
    trt.associated_type("Bar").bound("Clone").bound("Send");

    insta::assert_snapshot!(scope.to_string(), @r"
    trait Foo {
        type Bar: Clone + Send;
    }
    ");
}

#[test]
fn trait_with_attr() {
    let mut scope = Scope::new();
    scope
        .new_trait("Foo")
        .attr("deprecated");

    insta::assert_snapshot!(scope.to_string(), @r"
    #[deprecated]
    trait Foo {}
    ");
}

#[test]
fn trait_fn_without_body() {
    let mut scope = Scope::new();
    let trt = scope.new_trait("Foo");
    trt.new_fn("bar").arg_ref_self().ret("usize");

    insta::assert_snapshot!(scope.to_string(), @r"
    trait Foo {
        fn bar(&self) -> usize;
    }
    ");
}

#[test]
fn trait_fn_with_default_body() {
    let mut scope = Scope::new();
    let trt = scope.new_trait("Foo");
    let f = trt.new_fn("bar");
    f.arg_ref_self().ret("usize");
    f.body = Some(vec![]);
    f.line("42");

    insta::assert_snapshot!(scope.to_string(), @r"
    trait Foo {
        fn bar(&self) -> usize {
            42
        }
    }
    ");
}

// ── Impl ──────────────────────────────────────────────────────────────

#[test]
fn impl_without_trait() {
    let mut scope = Scope::new();
    scope.new_struct("Foo").field("x", "usize");
    let imp = scope.new_impl("Foo");
    imp.new_fn("new").ret("Self").line("Foo { x: 0 }");

    insta::assert_snapshot!(scope.to_string(), @r"
    struct Foo {
        x: usize,
    }

    impl Foo {
        fn new() -> Self {
            Foo { x: 0 }
        }
    }
    ");
}

#[test]
fn impl_with_generics_and_bounds() {
    let mut scope = Scope::new();
    let imp = scope.new_impl("Wrapper");
    imp.generic("T");
    imp.target_generic("T");
    imp.impl_trait("Display");
    imp.bound("T", "Display");
    imp.new_fn("fmt")
        .arg_ref_self()
        .arg("f", "&mut fmt::Formatter<'_>")
        .ret("fmt::Result")
        .line("write!(f, \"{}\", self.0)");

    insta::assert_snapshot!(scope.to_string(), @r#"
    impl<T> Display for Wrapper<T>
    where
        T: Display,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    "#);
}

#[test]
fn impl_with_associated_type() {
    let mut scope = Scope::new();
    let imp = scope.new_impl("MyIter");
    imp.impl_trait("Iterator");
    imp.associate_type("Item", "u32");
    imp.new_fn("next")
        .arg_mut_self()
        .ret("Option<Self::Item>")
        .line("None");

    insta::assert_snapshot!(scope.to_string(), @r"
    impl Iterator for MyIter {
        type Item = u32;

        fn next(&mut self) -> Option<Self::Item> {
            None
        }
    }
    ");
}

#[test]
fn enum_with_vis() {
    let mut scope = Scope::new();
    scope
        .new_enum("Color")
        .vis("pub")
        .push_variant(Variant::new("Red"))
        .push_variant(Variant::new("Blue"));

    insta::assert_snapshot!(scope.to_string(), @r"
    pub enum Color {
        Red,
        Blue,
    }
    ");
}

#[test]
fn enum_with_generics_and_bounds() {
    let mut scope = Scope::new();
    let enu = scope.new_enum("MyResult");
    enu.generic("T").generic("E").bound("E", "Debug");
    enu.new_variant("Ok").tuple("T");
    enu.new_variant("Err").tuple("E");

    insta::assert_snapshot!(scope.to_string(), @r"
    enum MyResult<T, E>
    where
        E: Debug,
    {
        Ok(T),
        Err(E),
    }
    ");
}

#[test]
fn enum_with_doc() {
    let mut scope = Scope::new();
    scope
        .new_enum("Dir")
        .doc("Cardinal directions.")
        .push_variant(Variant::new("North"))
        .push_variant(Variant::new("South"));

    insta::assert_snapshot!(scope.to_string(), @r"
    /// Cardinal directions.
    enum Dir {
        North,
        South,
    }
    ");
}

#[test]
fn enum_with_macro() {
    let mut scope = Scope::new();
    scope
        .new_enum("Msg")
        .r#macro("#[serde(tag = \"type\")]")
        .push_variant(Variant::new("Ping"))
        .push_variant(Variant::new("Pong"));

    insta::assert_snapshot!(scope.to_string(), @r#"
    #[serde(tag = "type")]
    enum Msg {
        Ping,
        Pong,
    }
    "#);
}

// ── Variant extras ────────────────────────────────────────────────────

#[test]
fn variant_with_annotation() {
    let mut scope = Scope::new();
    let enu = scope.new_enum("Foo");
    enu.new_variant("Bar")
        .annotation("#[serde(rename = \"bar\")]");
    enu.new_variant("Baz")
        .annotation("#[deprecated]")
        .annotation("#[doc(hidden)]");

    insta::assert_snapshot!(scope.to_string(), @r#"
    enum Foo {
        #[serde(rename = "bar")]
        Bar,
        #[deprecated]
        #[doc(hidden)]
        Baz,
    }
    "#);
}

#[test]
fn variant_with_named_fields() {
    let mut scope = Scope::new();
    let enu = scope.new_enum("Shape");
    enu.new_variant("Circle").named("radius", "f64");
    enu.new_variant("Rect")
        .named("width", "f64")
        .named("height", "f64");

    insta::assert_snapshot!(scope.to_string(), @r"
    enum Shape {
        Circle { radius: f64 },
        Rect { width: f64, height: f64 },
    }
    ");
}

#[test]
fn variant_push_named_field() {
    let mut scope = Scope::new();
    let enu = scope.new_enum("Ev");
    let v = enu.new_variant("Click");
    let mut field = Field::new("x", "i32");
    field.doc("X coordinate");
    v.push_named(field);

    insta::assert_snapshot!(scope.to_string(), @r"
    enum Ev {
        Click {
            /// X coordinate
            x: i32,
        },
    }
    ");
}

#[test]
fn variant_with_multiple_tuple_fields() {
    let mut scope = Scope::new();
    let enu = scope.new_enum("Pair");
    enu.new_variant("Both").tuple("A").tuple("B");

    insta::assert_snapshot!(scope.to_string(), @r"
    enum Pair {
        Both(A, B),
    }
    ");
}

#[test]
fn variant_tuple_with_attrs() {
    let mut scope = Scope::new();
    let enu = scope.new_enum("Wrapper");
    enu.new_variant("Inner")
        .tuple_with_attrs(["#[serde(transparent)]"], "String");

    insta::assert_snapshot!(scope.to_string(), @r#"
    enum Wrapper {
        Inner(#[serde(transparent)] String),
    }
    "#);
}

#[test]
fn struct_with_vis() {
    let mut scope = Scope::new();
    scope
        .new_struct("Foo")
        .vis("pub")
        .field("x", "usize");

    insta::assert_snapshot!(scope.to_string(), @r"
    pub struct Foo {
        x: usize,
    }
    ");
}

#[test]
fn struct_with_macro() {
    let mut scope = Scope::new();
    scope
        .new_struct("Foo")
        .r#macro("#[serde(rename_all = \"camelCase\")]")
        .field("my_field", "String");

    insta::assert_snapshot!(scope.to_string(), @r#"
    #[serde(rename_all = "camelCase")]
    struct Foo {
        my_field: String,
    }
    "#);
}

#[test]
fn struct_with_attr() {
    let mut scope = Scope::new();
    scope
        .new_struct("Foo")
        .attr("non_exhaustive")
        .field("x", "u32");

    insta::assert_snapshot!(scope.to_string(), @r"
    #[non_exhaustive]
    struct Foo {
        x: u32,
    }
    ");
}

#[test]
fn struct_with_tuple_fields() {
    let mut scope = Scope::new();
    scope
        .new_struct("Pair")
        .tuple_field("u32")
        .tuple_field("String");

    insta::assert_snapshot!(scope.to_string(), @"struct Pair(u32, String);");
}

#[test]
fn struct_new_field_returns_mutable_ref() {
    let mut scope = Scope::new();
    let s = scope.new_struct("Foo");
    s.new_field("x", "u32").vis("pub").doc("The x value");
    s.field("y", "u32");

    insta::assert_snapshot!(scope.to_string(), @r"
    struct Foo {
        /// The x value
        pub x: u32,
        y: u32,
    }
    ");
}

#[test]
fn scope_raw() {
    let mut scope = Scope::new();
    scope.raw("// This is a raw comment");
    scope.new_struct("Foo");

    insta::assert_snapshot!(scope.to_string(), @r"
    // This is a raw comment

    struct Foo;
    ");
}

#[test]
fn scope_import_with_vis() {
    let mut scope = Scope::new();
    scope.import("std::collections", "HashMap").vis("pub");
    scope.new_struct("Foo").field("map", "HashMap<String, String>");

    insta::assert_snapshot!(scope.to_string(), @r"
    pub use std::collections::HashMap;

    struct Foo {
        map: HashMap<String, String>,
    }
    ");
}

#[test]
fn scope_modules_iterator() {
    let mut scope = Scope::new();
    scope.new_module("alpha");
    scope.new_struct("NotAModule");
    scope.new_module("beta");

    let names: Vec<&str> = scope.modules().map(|(name, _)| name).collect();
    assert_eq!(names, vec!["alpha", "beta"]);
}

// ── Module extras ─────────────────────────────────────────────────────

#[test]
fn module_with_vis() {
    let mut scope = Scope::new();
    scope.new_module("inner").vis("pub").new_struct("Foo");

    insta::assert_snapshot!(scope.to_string(), @r"
    pub mod inner {
        struct Foo;
    }
    ");
}

#[test]
fn module_with_attr() {
    let mut scope = Scope::new();
    scope
        .new_module("tests")
        .attr("cfg(test)")
        .new_fn("it_works")
        .attr("test")
        .line("assert!(true);");

    insta::assert_snapshot!(scope.to_string(), @r"
    #[cfg(test)]
    mod tests {
        #[test]
        fn it_works() {
            assert!(true);
        }
    }
    ");
}

#[test]
fn nested_modules() {
    let mut scope = Scope::new();
    let outer = scope.new_module("outer");
    outer.new_module("inner").new_struct("Deep");

    insta::assert_snapshot!(scope.to_string(), @r"
    mod outer {
        mod inner {
            struct Deep;
        }
    }
    ");
}

#[test]
fn module_body_to_string() {
    let mut scope = Scope::new();
    let m = scope.new_module("my_mod");
    m.new_struct("Foo").field("x", "u32");

    let body = m.body_to_string();
    assert!(body.contains("struct Foo"));
    assert!(!body.contains("mod my_mod"));
}

#[test]
fn module_scope_access() {
    let mut scope = Scope::new();
    let m = scope.new_module("my_mod");
    m.scope().new_struct("ViaScope").field("a", "bool");

    insta::assert_snapshot!(scope.to_string(), @r"
    mod my_mod {
        struct ViaScope {
            a: bool,
        }
    }
    ");
}

#[test]
fn module_import_with_vis() {
    let mut scope = Scope::new();
    let m = scope.new_module("reexport");
    m.scope().import("std::fmt", "Display").vis("pub");
    m.new_struct("Foo");

    insta::assert_snapshot!(scope.to_string(), @r"
    mod reexport {
        pub use std::fmt::Display;

        struct Foo;
    }
    ");
}

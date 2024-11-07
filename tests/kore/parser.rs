use kframework::kore::parser::KoreParser;
use kframework::kore::syntax::{
    App, Definition, Id, Module, Pattern, SVar, Sentence, SetVarId, Sort, SymbolId, Var,
};

fn id<T: Into<String>>(s: T) -> Result<Id, String> {
    Id::new(s.into())
}

fn sym<T: Into<String>>(s: T) -> Result<SymbolId, String> {
    SymbolId::new(s.into())
}

fn svid<T: Into<String>>(s: T) -> Result<SetVarId, String> {
    SetVarId::new(s.into())
}

macro_rules! sort_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() -> Result<(), String> {
            // Given
            let (text, expected) = $value;
            let mut parser = KoreParser::new(text)?;

            // When
            let actual = parser.sort()?;

            // Then
            assert_eq!(expected, actual);
            Ok(())
        }
    )*
    }
}

sort_tests! {
    test_sort_var: ("S", Sort::Var(id("S")?)),
    test_sort_app: ("SortInt{}", Sort::App { id: id("SortInt")?, args: vec![] }),
    test_sort_app_with_params: (
        "SortMap{X, Y}",
        Sort::App {
            id: id("SortMap")?,
            args: vec![
                Sort::Var(id("X")?),
                Sort::Var(id("Y")?),
            ],
        }
    ),
}

macro_rules! pattern_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() -> Result<(), String> {
            // Given
            let (text, expected) = $value;
            let mut parser = KoreParser::new(text).unwrap();

            // When
            let actual = parser.pattern().unwrap();

            // Then
            assert_eq!(expected, actual);
            Ok(())
        }
    )*
    }
}

pattern_tests! {
    test_pattern_var: (
        "X : S",
        Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?)}),
    ),
    test_pattern_svar: (
        "@X : S",
        Pattern::SVar(SVar { id: svid("@X")?, sort: Sort::Var(id("S")?)}),
    ),
    test_pattern_str: (
        r#""a\u03b1\x80\U0001f642b""#,
        Pattern::Str("a\u{3b1}\u{80}\u{1f642}b".into()),
    ),
    test_pattern_app: (
        r"\foo{S, T}(X : S, Y : T)",
        Pattern::App(
            App {
                symbol: sym("\\foo")?,
                sorts: vec![Sort::Var(id("S")?), Sort::Var(id("T")?)],
                args: vec![
                    Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?) }),
                    Pattern::Var(Var { id: id("Y")?, sort: Sort::Var(id("T")?) }),
                ],
            },
        ),
    ),
    test_pattern_left_assoc: (
        r"\left-assoc{}(\foo{S, T}(X : S, Y : T))",
        Pattern::LeftAssoc(
            App {
                symbol: sym("\\foo")?,
                sorts: vec![Sort::Var(id("S")?), Sort::Var(id("T")?)],
                args: vec![
                    Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?) }),
                    Pattern::Var(Var { id: id("Y")?, sort: Sort::Var(id("T")?) }),
                ],
            },
        ),
    ),
    test_pattern_right_assoc: (
        r"\right-assoc{}(\foo{S, T}(X : S, Y : T))",
        Pattern::RightAssoc(
            App {
                symbol: sym("\\foo")?,
                sorts: vec![Sort::Var(id("S")?), Sort::Var(id("T")?)],
                args: vec![
                    Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?) }),
                    Pattern::Var(Var { id: id("Y")?, sort: Sort::Var(id("T")?) }),
                ],
            },
        ),
    ),
    test_pattern_top: (
        r"\top{S}()",
        Pattern::Top(Sort::Var(id("S")?)),
    ),
    test_pattern_bottom: (
        r"\bottom{S}()",
        Pattern::Bottom(Sort::Var(id("S")?)),
    ),
    test_dv: (
        r#"\dv{SortInt{}}("0")"#,
        Pattern::Dv {
            sort: Sort::App { id: id("SortInt")?, args: vec![] },
            value: "0".into(),
        },
    ),
    test_not: (
        r"\not{S}(X : S)",
        Pattern::Not {
            sort: Sort::Var(id("S")?),
            op: Box::new(Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?)})),
        },
    ),
    test_implies: (
        r"\implies{S}(X : S, Y : S)",
        Pattern::Implies {
            sort: Sort::Var(id("S")?),
            left: Box::new(Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?)})),
            right: Box::new(Pattern::Var(Var { id: id("Y")?, sort: Sort::Var(id("S")?)})),
        },
    ),
    test_iff: (
        r"\iff{S}(X : S, Y : S)",
        Pattern::Iff {
            sort: Sort::Var(id("S")?),
            left: Box::new(Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?)})),
            right: Box::new(Pattern::Var(Var { id: id("Y")?, sort: Sort::Var(id("S")?)})),
        },
    ),
    test_and_0: (
        r"\and{S}()",
        Pattern::And {
            sort: Sort::Var(id("S")?),
            ops: vec![],
        },
    ),
    test_and_1: (
        r"\and{S}(X : S)",
        Pattern::And {
            sort: Sort::Var(id("S")?),
            ops: vec![
                Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?) }),
            ],
        },
    ),
    test_and_2: (
        r"\and{S}(X : S, Y : S)",
        Pattern::And {
            sort: Sort::Var(id("S")?),
            ops: vec![
                Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?) }),
                Pattern::Var(Var { id: id("Y")?, sort: Sort::Var(id("S")?) }),
            ],
        },
    ),
    test_and_3: (
        r"\and{S}(X : S, Y : S, Z : S)",
        Pattern::And {
            sort: Sort::Var(id("S")?),
            ops: vec![
                Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?) }),
                Pattern::Var(Var { id: id("Y")?, sort: Sort::Var(id("S")?) }),
                Pattern::Var(Var { id: id("Z")?, sort: Sort::Var(id("S")?) }),
            ],
        },
    ),
    test_or_0: (
        r"\or{S}()",
        Pattern::Or {
            sort: Sort::Var(id("S")?),
            ops: vec![],
        },
    ),
    test_or_1: (
        r"\or{S}(X : S)",
        Pattern::Or {
            sort: Sort::Var(id("S")?),
            ops: vec![
                Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?) }),
            ],
        },
    ),
    test_or_2: (
        r"\or{S}(X : S, Y : S)",
        Pattern::Or {
            sort: Sort::Var(id("S")?),
            ops: vec![
                Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?) }),
                Pattern::Var(Var { id: id("Y")?, sort: Sort::Var(id("S")?) }),
            ],
        },
    ),
    test_or_3: (
        r"\or{S}(X : S, Y : S, Z : S)",
        Pattern::Or {
            sort: Sort::Var(id("S")?),
            ops: vec![
                Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?) }),
                Pattern::Var(Var { id: id("Y")?, sort: Sort::Var(id("S")?) }),
                Pattern::Var(Var { id: id("Z")?, sort: Sort::Var(id("S")?) }),
            ],
        },
    ),
    test_exists: (
        r"\exists{S}(X : S, Y : S)",
        Pattern::Exists {
            sort: Sort::Var(id("S")?),
            var: Var { id: id("X")?, sort: Sort::Var(id("S")?) },
            op: Box::new(Pattern::Var(Var { id: id("Y")?, sort: Sort::Var(id("S")?) })),
        },
    ),
    test_forall: (
        r"\forall{S}(X : S, Y : S)",
        Pattern::Forall {
            sort: Sort::Var(id("S")?),
            var: Var { id: id("X")?, sort: Sort::Var(id("S")?) },
            op: Box::new(Pattern::Var(Var { id: id("Y")?, sort: Sort::Var(id("S")?) })),
        },
    ),
    test_mu: (
        r"\mu{}(@X : S, Y : S)",
        Pattern::Mu {
            var: SVar { id: svid("@X")?, sort: Sort::Var(id("S")?) },
            op: Box::new(Pattern::Var(Var { id: id("Y")?, sort: Sort::Var(id("S")?) })),
        }
    ),
    test_nu: (
        r"\nu{}(@X : S, Y : S)",
        Pattern::Nu {
            var: SVar { id: svid("@X")?, sort: Sort::Var(id("S")?) },
            op: Box::new(Pattern::Var(Var { id: id("Y")?, sort: Sort::Var(id("S")?) })),
        }
    ),
    test_ceil: (
        r"\ceil{S, T}(X : S)",
        Pattern::Ceil {
            op_sort: Sort::Var(id("S")?),
            sort: Sort::Var(id("T")?),
            op: Box::new(Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?) })),
        }
    ),
    test_floor: (
        r"\floor{S, T}(X : S)",
        Pattern::Floor {
            op_sort: Sort::Var(id("S")?),
            sort: Sort::Var(id("T")?),
            op: Box::new(Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?) })),
        }
    ),
    test_equals: (
        r"\equals{S, T}(X : S, Y : S)",
        Pattern::Equals {
            op_sort: Sort::Var(id("S")?),
            sort: Sort::Var(id("T")?),
            left: Box::new(Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?) })),
            right: Box::new(Pattern::Var(Var { id: id("Y")?, sort: Sort::Var(id("S")?) })),
        },
    ),
    test_in: (
        r"\in{S, T}(X : S, Y : S)",
        Pattern::In {
            op_sort: Sort::Var(id("S")?),
            sort: Sort::Var(id("T")?),
            left: Box::new(Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?) })),
            right: Box::new(Pattern::Var(Var { id: id("Y")?, sort: Sort::Var(id("S")?) })),
        },
    ),
    test_next: (
        r"\next{S}(X : S)",
        Pattern::Next {
            sort: Sort::Var(id("S")?),
            op: Box::new(Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?) })),
        },
    ),
    test_rewrites: (
        r"\rewrites{S}(X : S, Y : S)",
        Pattern::Rewrites {
            sort: Sort::Var(id("S")?),
            left: Box::new(Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?) })),
            right: Box::new(Pattern::Var(Var { id: id("Y")?, sort: Sort::Var(id("S")?) })),
        },
    ),
}

macro_rules! sentence_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() -> Result<(), String> {
            // Given
            let (text, expected) = $value;
            let mut parser = KoreParser::new(text)?;

            // When
            let actual = parser.sentence()?;

            // Then
            assert_eq!(expected, actual);
            Ok(())
        }
    )*
    }
}

sentence_tests! {
    test_sentence_import: (
        "import FOO []",
        Sentence::Import {
            module: id("FOO")?,
            attrs: vec![],
        },
    ),
    test_sentence_sort: (
        "sort SortFoo{S, T} []",
        Sentence::Sort {
            id: id("SortFoo")?,
            vars: vec![id("S")?, id("T")?],
            attrs: vec![],
            hooked: false,
        },
    ),
    test_sentence_hooked_sort: (
        "hooked-sort SortFoo{S, T} []",
        Sentence::Sort {
            id: id("SortFoo")?,
            vars: vec![id("S")?, id("T")?],
            attrs: vec![],
            hooked: true,
        },
    ),
    test_sentence_symbol: (
        r"symbol \foo{S, T}(S, SortInt{}): T []",
        Sentence::Symbol {
            id: sym(r"\foo")?,
            vars: vec![id("S")?, id("T")?],
            param_sorts: vec![
                Sort::Var(id("S")?),
                Sort::App { id: id("SortInt")?, args: vec![] },
            ],
            sort: Sort::Var(id("T")?),
            attrs: vec![],
            hooked: false,
        },
    ),
    test_sentence_hooked_symbol: (
        r"hooked-symbol \foo{S, T}(S, SortInt{}): T []",
        Sentence::Symbol {
            id: sym(r"\foo")?,
            vars: vec![id("S")?, id("T")?],
            param_sorts: vec![
                Sort::Var(id("S")?),
                Sort::App { id: id("SortInt")?, args: vec![] },
            ],
            sort: Sort::Var(id("T")?),
            attrs: vec![],
            hooked: true,
        },
    ),
    test_alias: (
        r"alias \foo{S}(S) : SortBool{} where \foo{S}(X : S) := X : S []",
        Sentence::Alias {
            id: sym(r"\foo")?,
            vars: vec![id("S")?],
            param_sorts: vec![Sort::Var(id("S")?)],
            sort: Sort::App { id: id("SortBool")?, args: vec![] },
            left: App {
                symbol: sym(r"\foo")?,
                sorts: vec![Sort::Var(id("S")?)],
                args: vec![Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?) })],
            },
            right: Box::new(
                Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?)}),
            ),
            attrs: vec![],
        },
    ),
    test_axiom: (
        "axiom{S} X : S []",
        Sentence::Axiom {
            vars: vec![id("S")?],
            pattern: Box::new(Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?) })),
            attrs: vec![],
        },
    ),
    test_claim: (
        "claim{S} X : S []",
        Sentence::Claim {
            vars: vec![id("S")?],
            pattern: Box::new(Pattern::Var(Var { id: id("X")?, sort: Sort::Var(id("S")?) })),
            attrs: vec![],
        },
    ),
}

#[test]
fn test_definition() -> Result<(), String> {
    // Given
    let text = r#"
        [foo{}("bar"), baz{}()]

        module MODULE-SYNTAX
            sort S{} []
        endmodule []

        module MODULE
            import MODULE-SYNTAX []
        endmodule []
    "#;
    let expected = Definition {
        modules: vec![
            Module {
                id: id("MODULE-SYNTAX")?,
                sentences: vec![Sentence::Sort {
                    id: id("S")?,
                    vars: vec![],
                    attrs: vec![],
                    hooked: false,
                }],
                attrs: vec![],
            },
            Module {
                id: id("MODULE")?,
                sentences: vec![Sentence::Import {
                    module: id("MODULE-SYNTAX")?,
                    attrs: vec![],
                }],
                attrs: vec![],
            },
        ],
        attrs: vec![
            App {
                symbol: sym("foo")?,
                sorts: vec![],
                args: vec![Pattern::Str("bar".into())],
            },
            App {
                symbol: sym("baz")?,
                sorts: vec![],
                args: vec![],
            },
        ],
    };
    let mut parser = KoreParser::new(text)?;

    // When
    let actual = parser.definition()?;

    // Then
    assert_eq!(expected, actual);
    Ok(())
}

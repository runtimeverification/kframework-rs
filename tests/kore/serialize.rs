use indoc::indoc;
use serde_json;

use kframework::kore::Parser;

macro_rules! sort_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() -> Result<(), String> {
            // Given
            let (text, expected) = $value;
            let sort = Parser::new(text)?.sort()?;

            // When
            let actual = serde_json::to_string_pretty(&sort).map_err(|e| e.to_string())?;

            // Then
            assert_eq!(expected, actual);
            Ok(())
        }
    )*
    }
}

sort_tests! {
    test_sort_var: (
        "S",
        indoc! {r#"{
          "tag": "SortVar",
          "name": "S"
        }"#},
    ),
    test_sort_app: (
        "SortFoo{S, T}",
        indoc! {r#"{
          "tag": "SortApp",
          "name": "SortFoo",
          "args": [
            {
              "tag": "SortVar",
              "name": "S"
            },
            {
              "tag": "SortVar",
              "name": "T"
            }
          ]
        }"#},
    ),
}

macro_rules! pattern_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() -> Result<(), String> {
            // Given
            let (text, expected) = $value;
            let sort = Parser::new(text)?.pattern()?;

            // When
            let actual = serde_json::to_string_pretty(&sort).map_err(|e| e.to_string())?;

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
        indoc! {r#"{
          "tag": "EVar",
          "name": "X",
          "sort": {
            "tag": "SortVar",
            "name": "S"
          }
        }"#},
    ),
    test_pattern_svar: (
        "@X : S",
        indoc! {r#"{
          "tag": "SVar",
          "name": "@X",
          "sort": {
            "tag": "SortVar",
            "name": "S"
          }
        }"#},
    ),
    test_pattern_str: (
        r#""foo\x80""#,
        indoc! {"{
          \"tag\": \"String\",
          \"value\": \"foo\u{80}\"
        }"},
    ),
    test_pattern_app: (
        r"\foo{S}(X : S)",
        indoc! {r#"{
          "tag": "App",
          "name": "\\foo",
          "sorts": [
            {
              "tag": "SortVar",
              "name": "S"
            }
          ],
          "args": [
            {
              "tag": "EVar",
              "name": "X",
              "sort": {
                "tag": "SortVar",
                "name": "S"
              }
            }
          ]
        }"#},
    ),
    test_pattern_left_assoc: (
        r"\left-assoc{}(\foo{S}(X : S))",
        indoc! {r#"{
          "tag": "LeftAssoc",
          "symbol": "\\foo",
          "sorts": [
            {
              "tag": "SortVar",
              "name": "S"
            }
          ],
          "argss": [
            {
              "tag": "EVar",
              "name": "X",
              "sort": {
                "tag": "SortVar",
                "name": "S"
              }
            }
          ]
        }"#},
    ),
    test_pattern_right_assoc: (
        r"\right-assoc{}(\foo{S}(X : S))",
        indoc! {r#"{
          "tag": "RightAssoc",
          "symbol": "\\foo",
          "sorts": [
            {
              "tag": "SortVar",
              "name": "S"
            }
          ],
          "argss": [
            {
              "tag": "EVar",
              "name": "X",
              "sort": {
                "tag": "SortVar",
                "name": "S"
              }
            }
          ]
        }"#},
    ),
    test_pattern_top: (
        r"\top{S}()",
        indoc! {r#"{
          "tag": "Top",
          "sort": {
            "tag": "SortVar",
            "name": "S"
          }
        }"#},
    ),
    test_pattern_bottom: (
        r"\bottom{S}()",
        indoc! {r#"{
          "tag": "Bottom",
          "sort": {
            "tag": "SortVar",
            "name": "S"
          }
        }"#},
    ),
    test_pattern_dv: (
        r#"\dv{SortInt{}}("0")"#,
        indoc! {r#"{
          "tag": "DV",
          "sort": {
            "tag": "SortApp",
            "name": "SortInt",
            "args": []
          },
          "value": "0"
        }"#},
    ),
    test_pattern_not: (
        r#"\not{S}(X : S)"#,
        indoc! {r#"{
          "tag": "Not",
          "sort": {
            "tag": "SortVar",
            "name": "S"
          },
          "arg": {
            "tag": "EVar",
            "name": "X",
            "sort": {
              "tag": "SortVar",
              "name": "S"
            }
          }
        }"#},
    ),
    test_pattern_implies: (
        r#"\implies{S}(X : S, Y : S)"#,
        indoc! {r#"{
          "tag": "Implies",
          "sort": {
            "tag": "SortVar",
            "name": "S"
          },
          "first": {
            "tag": "EVar",
            "name": "X",
            "sort": {
              "tag": "SortVar",
              "name": "S"
            }
          },
          "second": {
            "tag": "EVar",
            "name": "Y",
            "sort": {
              "tag": "SortVar",
              "name": "S"
            }
          }
        }"#},
    ),
    test_pattern_iff: (
        r#"\iff{S}(X : S, Y : S)"#,
        indoc! {r#"{
          "tag": "Iff",
          "sort": {
            "tag": "SortVar",
            "name": "S"
          },
          "first": {
            "tag": "EVar",
            "name": "X",
            "sort": {
              "tag": "SortVar",
              "name": "S"
            }
          },
          "second": {
            "tag": "EVar",
            "name": "Y",
            "sort": {
              "tag": "SortVar",
              "name": "S"
            }
          }
        }"#},
    ),
    test_pattern_and: (
        r#"\and{S}(X : S)"#,
        indoc! {r#"{
          "tag": "And",
          "sort": {
            "tag": "SortVar",
            "name": "S"
          },
          "patterns": [
            {
              "tag": "EVar",
              "name": "X",
              "sort": {
                "tag": "SortVar",
                "name": "S"
              }
            }
          ]
        }"#},
    ),
    test_pattern_or: (
        r#"\or{S}(X : S)"#,
        indoc! {r#"{
          "tag": "Or",
          "sort": {
            "tag": "SortVar",
            "name": "S"
          },
          "patterns": [
            {
              "tag": "EVar",
              "name": "X",
              "sort": {
                "tag": "SortVar",
                "name": "S"
              }
            }
          ]
        }"#},
    ),
    test_pattern_exists: (
        r#"\exists{S}(X : S, Y : S)"#,
        indoc! {r#"{
          "tag": "Exists",
          "sort": {
            "tag": "SortVar",
            "name": "S"
          },
          "var": "X",
          "varSort": {
            "tag": "SortVar",
            "name": "S"
          },
          "arg": {
            "tag": "EVar",
            "name": "Y",
            "sort": {
              "tag": "SortVar",
              "name": "S"
            }
          }
        }"#},
    ),
    test_pattern_forall: (
        r#"\forall{S}(X : S, Y : S)"#,
        indoc! {r#"{
          "tag": "Forall",
          "sort": {
            "tag": "SortVar",
            "name": "S"
          },
          "var": "X",
          "varSort": {
            "tag": "SortVar",
            "name": "S"
          },
          "arg": {
            "tag": "EVar",
            "name": "Y",
            "sort": {
              "tag": "SortVar",
              "name": "S"
            }
          }
        }"#},
    ),
    test_pattern_mu: (
        r#"\mu{}(@X : S, Y : S)"#,
        indoc! {r#"{
          "tag": "Mu",
          "var": "@X",
          "varSort": {
            "tag": "SortVar",
            "name": "S"
          },
          "arg": {
            "tag": "EVar",
            "name": "Y",
            "sort": {
              "tag": "SortVar",
              "name": "S"
            }
          }
        }"#},
    ),
    test_pattern_nu: (
        r#"\nu{}(@X : S, Y : S)"#,
        indoc! {r#"{
          "tag": "Nu",
          "var": "@X",
          "varSort": {
            "tag": "SortVar",
            "name": "S"
          },
          "arg": {
            "tag": "EVar",
            "name": "Y",
            "sort": {
              "tag": "SortVar",
              "name": "S"
            }
          }
        }"#},
    ),
    test_pattern_ceil: (
        r#"\ceil{S, T}(X : S)"#,
        indoc! {r#"{
          "tag": "Ceil",
          "argSort": {
            "tag": "SortVar",
            "name": "S"
          },
          "sort": {
            "tag": "SortVar",
            "name": "T"
          },
          "arg": {
            "tag": "EVar",
            "name": "X",
            "sort": {
              "tag": "SortVar",
              "name": "S"
            }
          }
        }"#},
    ),
    test_pattern_floor: (
        r#"\floor{S, T}(X : S)"#,
        indoc! {r#"{
          "tag": "Floor",
          "argSort": {
            "tag": "SortVar",
            "name": "S"
          },
          "sort": {
            "tag": "SortVar",
            "name": "T"
          },
          "arg": {
            "tag": "EVar",
            "name": "X",
            "sort": {
              "tag": "SortVar",
              "name": "S"
            }
          }
        }"#},
    ),
    test_pattern_equals: (
        r#"\equals{S, T}(X : S, Y : S)"#,
        indoc! {r#"{
          "tag": "Equals",
          "argSort": {
            "tag": "SortVar",
            "name": "S"
          },
          "sort": {
            "tag": "SortVar",
            "name": "T"
          },
          "first": {
            "tag": "EVar",
            "name": "X",
            "sort": {
              "tag": "SortVar",
              "name": "S"
            }
          },
          "second": {
            "tag": "EVar",
            "name": "Y",
            "sort": {
              "tag": "SortVar",
              "name": "S"
            }
          }
        }"#},
    ),
    test_pattern_in: (
        r#"\in{S, T}(X : S, Y : S)"#,
        indoc! {r#"{
          "tag": "In",
          "argSort": {
            "tag": "SortVar",
            "name": "S"
          },
          "sort": {
            "tag": "SortVar",
            "name": "T"
          },
          "first": {
            "tag": "EVar",
            "name": "X",
            "sort": {
              "tag": "SortVar",
              "name": "S"
            }
          },
          "second": {
            "tag": "EVar",
            "name": "Y",
            "sort": {
              "tag": "SortVar",
              "name": "S"
            }
          }
        }"#},
    ),
    test_pattern_next: (
        r#"\next{S}(X : S)"#,
        indoc! {r#"{
          "tag": "Next",
          "sort": {
            "tag": "SortVar",
            "name": "S"
          },
          "dest": {
            "tag": "EVar",
            "name": "X",
            "sort": {
              "tag": "SortVar",
              "name": "S"
            }
          }
        }"#},
    ),
    test_pattern_rewrites: (
        r#"\rewrites{S}(X : S, Y : S)"#,
        indoc! {r#"{
          "tag": "Rewrites",
          "sort": {
            "tag": "SortVar",
            "name": "S"
          },
          "source": {
            "tag": "EVar",
            "name": "X",
            "sort": {
              "tag": "SortVar",
              "name": "S"
            }
          },
          "dest": {
            "tag": "EVar",
            "name": "Y",
            "sort": {
              "tag": "SortVar",
              "name": "S"
            }
          }
        }"#},
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Id(pub(crate) String);

impl Id {
    pub fn new(s: String) -> Result<Self, String> {
        let mut first_checked = false;
        for c in s.chars() {
            if !first_checked {
                if !c.is_ascii_alphabetic() {
                    return Err(format!("Expected alphabetic character, got: {:?}", c));
                }
                first_checked = true;
            } else if !c.is_ascii_alphanumeric() && c != '\'' && c != '-' {
                return Err(format!(
                    "Expected alphanumeric character, '\'' or '-', got: {:?}",
                    c
                ));
            }
        }
        if first_checked {
            Ok(Id(s))
        } else {
            Err(String::from("Invalid identifier: empty string"))
        }
    }

    pub fn value(self) -> String {
        self.0
    }
}

impl TryFrom<String> for Id {
    type Error = String;

    fn try_from(s: String) -> Result<Self, String> {
        Id::new(s)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolId(pub(crate) String);

impl SymbolId {
    pub fn new(s: String) -> Result<Self, String> {
        enum State {
            First,
            Second,
            Rest,
        }
        let mut state = State::First;
        for c in s.chars() {
            match state {
                State::First => {
                    if c == '\\' {
                        state = State::Second;
                    } else if c.is_ascii_alphabetic() {
                        state = State::Rest;
                    } else {
                        return Err(format!(
                            "Expected alphabetic character or '\\', got: {:?}",
                            c
                        ));
                    }
                }
                State::Second => {
                    if !c.is_ascii_alphabetic() {
                        return Err(format!("Expected alphabetic character, got: {:?}", c));
                    }
                    state = State::Rest;
                }
                State::Rest => {
                    if !c.is_ascii_alphanumeric() && c != '\'' && c != '-' {
                        return Err(format!(
                            "Expected alphanumeric character, '\'' or '-', got: {:?}",
                            c
                        ));
                    }
                }
            }
        }
        match state {
            State::First | State::Second => Err(format!("Invalid symbol: {:?}", s)),
            State::Rest => Ok(SymbolId(s)),
        }
    }

    pub fn value(self) -> String {
        self.0
    }
}

impl TryFrom<String> for SymbolId {
    type Error = String;

    fn try_from(s: String) -> Result<Self, String> {
        SymbolId::new(s)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetVarId(pub(crate) String);

impl SetVarId {
    pub fn new(s: String) -> Result<Self, String> {
        enum State {
            First,
            Second,
            Rest,
        }
        let mut state = State::First;
        for c in s.chars() {
            match state {
                State::First => {
                    if c != '@' {
                        return Err(format!("Expected '@', got: {:?}", c));
                    }
                    state = State::Second;
                }
                State::Second => {
                    if !c.is_ascii_alphabetic() {
                        return Err(format!("Expected alphabetic character, got: {:?}", c));
                    }
                    state = State::Rest;
                }
                State::Rest => {
                    if !c.is_ascii_alphanumeric() && c != '\'' && c != '-' {
                        return Err(format!(
                            "Expected alphanumeric character, '\'' or '-', got: {:?}",
                            c
                        ));
                    }
                }
            }
        }
        match state {
            State::First | State::Second => Err(format!("Invalid set variable: {:?}", s)),
            State::Rest => Ok(SetVarId(s)),
        }
    }

    pub fn value(self) -> String {
        self.0
    }
}

impl TryFrom<String> for SetVarId {
    type Error = String;

    fn try_from(s: String) -> Result<Self, String> {
        SetVarId::new(s)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Str(pub String);

impl Str {
    pub fn from_kore(s: &str) -> Result<Self, String> {
        let mut chars: Vec<char> = Vec::new();

        enum State {
            Normal,
            Escape,
            CodePoint,
        }

        let mut state = State::Normal;
        let mut acc = 0;
        let mut cnt = 0;

        for c in s.chars() {
            match state {
                State::Normal => {
                    if c == '\\' {
                        state = State::Escape;
                    } else {
                        chars.push(c);
                    }
                }
                State::Escape => {
                    let escaped = match c {
                        '"' => Some('"'),
                        '\\' => Some('\\'),
                        'f' => Some('\x0c'),
                        'n' => Some('\n'),
                        'r' => Some('\r'),
                        't' => Some('\t'),
                        _ => None,
                    };
                    if let Some(escaped) = escaped {
                        chars.push(escaped);
                        state = State::Normal;
                        continue;
                    }
                    cnt = match c {
                        'x' => 2,
                        'u' => 4,
                        'U' => 8,
                        _ => return Err(format!("Unexpected escape sequence: \\{}", c)),
                    };
                    state = State::CodePoint;
                }
                State::CodePoint => {
                    let Some(digit) = c.to_digit(16) else {
                        return Err(format!("Invalid hex digit: {:?}", c));
                    };
                    acc = 16 * acc + digit;
                    cnt -= 1;
                    if cnt == 0 {
                        let Some(encoded) = char::from_u32(acc) else {
                            return Err(format!("Invalid unicode code point: {:x}", acc));
                        };
                        chars.push(encoded);
                        acc = 0;
                        state = State::Normal;
                    }
                }
            }
        }

        Ok(Str(chars.into_iter().collect()))
    }
}

impl<T: Into<String>> From<T> for Str {
    fn from(s: T) -> Self {
        Str(s.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Var {
    pub id: Id,
    pub sort: Sort,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SVar {
    pub id: SetVarId,
    pub sort: Sort,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct App {
    pub symbol: SymbolId,
    pub sorts: Vec<Sort>,
    pub args: Vec<Pattern>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Definition {
    pub modules: Vec<Module>,
    pub attrs: Vec<App>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Module {
    pub id: Id,
    pub sentences: Vec<Sentence>,
    pub attrs: Vec<App>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sentence {
    Import {
        module: Id,
        attrs: Vec<App>,
    },
    Sort {
        id: Id,
        vars: Vec<Id>,
        attrs: Vec<App>,
        hooked: bool,
    },
    Symbol {
        id: SymbolId,
        vars: Vec<Id>,
        param_sorts: Vec<Sort>,
        sort: Sort,
        attrs: Vec<App>,
        hooked: bool,
    },
    Alias {
        id: SymbolId,
        vars: Vec<Id>,
        param_sorts: Vec<Sort>,
        sort: Sort,
        left: App,
        right: Box<Pattern>,
        attrs: Vec<App>,
    },
    Axiom {
        vars: Vec<Id>,
        pattern: Box<Pattern>,
        attrs: Vec<App>,
    },
    Claim {
        vars: Vec<Id>,
        pattern: Box<Pattern>,
        attrs: Vec<App>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sort {
    Var(Id),
    App { id: Id, args: Vec<Sort> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pattern {
    Var(Var),
    SVar(SVar),
    Str(Str),
    App(App),
    LeftAssoc(App),
    RightAssoc(App),
    Top(Sort),
    Bottom(Sort),
    Dv {
        sort: Sort,
        value: Str,
    },
    Not {
        sort: Sort,
        op: Box<Pattern>,
    },
    Implies {
        sort: Sort,
        left: Box<Pattern>,
        right: Box<Pattern>,
    },
    Iff {
        sort: Sort,
        left: Box<Pattern>,
        right: Box<Pattern>,
    },
    And {
        sort: Sort,
        ops: Vec<Pattern>,
    },
    Or {
        sort: Sort,
        ops: Vec<Pattern>,
    },
    Exists {
        sort: Sort,
        var: Var,
        op: Box<Pattern>,
    },
    Forall {
        sort: Sort,
        var: Var,
        op: Box<Pattern>,
    },
    Mu {
        var: SVar,
        op: Box<Pattern>,
    },
    Nu {
        var: SVar,
        op: Box<Pattern>,
    },
    Ceil {
        op_sort: Sort,
        sort: Sort,
        op: Box<Pattern>,
    },
    Floor {
        op_sort: Sort,
        sort: Sort,
        op: Box<Pattern>,
    },
    Equals {
        op_sort: Sort,
        sort: Sort,
        left: Box<Pattern>,
        right: Box<Pattern>,
    },
    In {
        op_sort: Sort,
        sort: Sort,
        left: Box<Pattern>,
        right: Box<Pattern>,
    },
    Next {
        sort: Sort,
        op: Box<Pattern>,
    },
    Rewrites {
        sort: Sort,
        left: Box<Pattern>,
        right: Box<Pattern>,
    },
}

#[cfg(test)]
mod tests {
    use super::Str;

    macro_rules! str_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() -> Result<(), String> {
                // Given
                let (input, expected) = $value;

                // When
                let actual = Str::from_kore(input)?.0;

                // Then
                assert_eq!(expected, actual);
                Ok(())
            }
        )*
        }
    }

    str_tests! {
        test_str_00: ("", ""),
        test_str_01: (" ", " "),
        test_str_02: ("foo", "foo"),
        test_str_03: (r"\f", "\x0c"),
        test_str_04: (r"\n", "\n"),
        test_str_05: (r"\r", "\r"),
        test_str_06: (r"\t", "\t"),
        test_str_07: (r"\\", "\\"),
        test_str_08: (r#"\""#, "\""),
        test_str_09: (r#"\""#, "\""),
        test_str_10: (r"\x80", "\u{80}"),
        test_str_11: (r"\x0f", "\u{f}"),
        test_str_12: (r"\x0F", "\u{f}"),
        test_str_13: (r"\u03b1", "\u{3b1}"),
        test_str_14: (r"\u03B1", "\u{3b1}"),
        test_str_15: (r"\U0001f642", "\u{1f642}"),
        test_str_16: (r"\U0001F642", "\u{1f642}"),
        test_str_17: (r"\x80\x80", "\u{80}\u{80}"),
        test_str_18: (r"a\u03b1\x80\U0001f642b", "a\u{3b1}\u{80}\u{1f642}b"),
    }
}

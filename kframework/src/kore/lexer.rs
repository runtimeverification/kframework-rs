use crate::error::KError::{self, KoreLexerError};
use std::str::Chars;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenType {
    Eof,
    Comma,
    Colon,
    Walrus,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBrack,
    RBrack,
    Str,
    Id,
    SymbolId,
    SetVarId,
    MlTop,
    MlBottom,
    MlNot,
    MlAnd,
    MlOr,
    MlImplies,
    MlIff,
    MlExists,
    MlForall,
    MlMu,
    MlNu,
    MlCeil,
    MlFloor,
    MlEquals,
    MlIn,
    MlNext,
    MlRewrites,
    MlDv,
    MlLeftAssoc,
    MlRightAssoc,
    KwModule,
    KwEndmodule,
    KwImport,
    KwSort,
    KwHookedSort,
    KwSymbol,
    KwHookedSymbol,
    KwAxiom,
    KwClaim,
    KwAlias,
    KwWhere,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'a> {
    ty: TokenType,
    text: &'a str,
    offset: usize,
}

impl<'a> Token<'a> {
    pub fn ty(&self) -> TokenType {
        self.ty
    }

    pub fn text(&self) -> &'a str {
        self.text
    }
}

#[derive(Debug)]
pub struct Lexer<'a> {
    text: &'a str,
    it: Chars<'a>,
    la: Option<char>,
    offset: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        let mut it = text.chars();
        let la = it.next();
        Self {
            text,
            it,
            la,
            offset: 0,
        }
    }

    pub fn next_token(&mut self) -> Result<Token<'a>, KError> {
        let la = loop {
            let Some(la) = self.la else {
                return Ok(Token {
                    ty: TokenType::Eof,
                    text: "",
                    offset: self.offset,
                });
            };
            match la {
                ' ' | '\n' | '\r' | '\t' | '\x0c' => self.consume_whitespace(),
                '/' => self.consume_comment()?,
                _ => break la,
            }
        };
        let token = match la {
            // Alpha
            'a'..='z' | 'A'..='Z' => self.id_or_keyword(),
            // Special symbols
            '"' => self.string()?,
            '@' => self.set_var_id()?,
            '\\' => self.symbol_or_ml_conn()?,
            ':' => self.colon_or_walrus(),
            // Tokens matching a single character
            ',' => self.comma(),
            '(' => self.lparen(),
            ')' => self.rparen(),
            '{' => self.lbrace(),
            '}' => self.rbrace(),
            '[' => self.lbrack(),
            ']' => self.rbrack(),
            // Error
            _ => return Err(KoreLexerError(format!("Unexpected character: {:?}", la))),
        };
        debug_assert!(token.text == &self.text[token.offset..token.offset + token.text.len()]);
        Ok(token)
    }

    fn colon_or_walrus(&mut self) -> Token<'a> {
        debug_assert!(self.la == Some(':'));
        let offset = self.offset;
        self.consume();
        let (ty, text) = if self.la == Some('=') {
            self.consume();
            (TokenType::Walrus, &self.text[offset..offset + 2])
        } else {
            (TokenType::Colon, &self.text[offset..offset + 1])
        };
        Token { ty, text, offset }
    }

    fn string(&mut self) -> Result<Token<'a>, KError> {
        debug_assert!(self.la == Some('"'));
        let offset = self.offset;
        let mut len: usize = 1;

        self.consume();
        loop {
            let la = self.la_or_err()?;
            len += 1;
            self.consume();
            match la {
                '\\' => {
                    len += 1;
                    self.consume();
                }
                '"' => break,
                _ => {}
            }
        }

        let text = &self.text[offset..offset + len];
        Ok(Token {
            ty: TokenType::Str,
            text,
            offset,
        })
    }

    fn id_or_keyword(&mut self) -> Token<'a> {
        debug_assert!(self.la.unwrap().is_ascii_alphabetic());
        let offset = self.offset;
        let mut len: usize = 1;

        self.consume();
        while let Some(la) = self.la {
            if la.is_ascii_alphanumeric() || la == '\'' || la == '-' {
                len += 1;
                self.consume();
            } else {
                break;
            }
        }

        let text = &self.text[offset..offset + len];
        let ty = match text {
            "module" => TokenType::KwModule,
            "endmodule" => TokenType::KwEndmodule,
            "import" => TokenType::KwImport,
            "sort" => TokenType::KwSort,
            "hooked-sort" => TokenType::KwHookedSort,
            "symbol" => TokenType::KwSymbol,
            "hooked-symbol" => TokenType::KwHookedSymbol,
            "axiom" => TokenType::KwAxiom,
            "claim" => TokenType::KwClaim,
            "alias" => TokenType::KwAlias,
            "where" => TokenType::KwWhere,
            _ => TokenType::Id,
        };
        Token { ty, text, offset }
    }

    fn symbol_or_ml_conn(&mut self) -> Result<Token<'a>, KError> {
        debug_assert!(self.la == Some('\\'));

        let offset = self.offset;
        let mut len: usize = 2;

        self.consume();
        let la = self.la_or_err()?;
        if !la.is_ascii_alphabetic() {
            return Err(KoreLexerError(format!("Expected letter, got: {:?}", la)));
        }

        self.consume();
        while let Some(la) = self.la {
            if la.is_ascii_alphanumeric() || la == '\'' || la == '-' {
                len += 1;
                self.consume();
            } else {
                break;
            }
        }

        let text = &self.text[offset..offset + len];
        let ty = match text {
            r"\top" => TokenType::MlTop,
            r"\bottom" => TokenType::MlBottom,
            r"\not" => TokenType::MlNot,
            r"\and" => TokenType::MlAnd,
            r"\or" => TokenType::MlOr,
            r"\implies" => TokenType::MlImplies,
            r"\iff" => TokenType::MlIff,
            r"\exists" => TokenType::MlExists,
            r"\forall" => TokenType::MlForall,
            r"\mu" => TokenType::MlMu,
            r"\nu" => TokenType::MlNu,
            r"\ceil" => TokenType::MlCeil,
            r"\floor" => TokenType::MlFloor,
            r"\equals" => TokenType::MlEquals,
            r"\in" => TokenType::MlIn,
            r"\next" => TokenType::MlNext,
            r"\rewrites" => TokenType::MlRewrites,
            r"\dv" => TokenType::MlDv,
            r"\left-assoc" => TokenType::MlLeftAssoc,
            r"\right-assoc" => TokenType::MlRightAssoc,
            _ => TokenType::SymbolId,
        };
        Ok(Token { ty, text, offset })
    }

    fn set_var_id(&mut self) -> Result<Token<'a>, KError> {
        debug_assert!(self.la == Some('@'));

        let offset = self.offset;
        let mut len: usize = 2;

        // TODO consume_alphabetic
        self.consume();
        let la = self.la_or_err()?;
        if !la.is_ascii_alphabetic() {
            return Err(KoreLexerError(format!("Expected letter, got: {:?}", la)));
        }

        // TODO consume_while or consume_until
        self.consume();
        while let Some(la) = self.la {
            if la.is_ascii_alphanumeric() || la == '\'' || la == '-' {
                len += 1;
                self.consume();
            } else {
                break;
            }
        }

        let text = &self.text[offset..offset + len];
        Ok(Token {
            ty: TokenType::SetVarId,
            text,
            offset,
        })
    }

    // TODO macro
    fn comma(&mut self) -> Token<'a> {
        debug_assert!(self.la == Some(','));
        let offset = self.offset;
        let text = &self.text[offset..offset + 1];
        debug_assert!(text == ",");
        self.consume();
        Token {
            ty: TokenType::Comma,
            text,
            offset,
        }
    }

    fn lparen(&mut self) -> Token<'a> {
        debug_assert!(self.la == Some('('));
        let offset = self.offset;
        let text = &self.text[offset..offset + 1];
        debug_assert!(text == "(");
        self.consume();
        Token {
            ty: TokenType::LParen,
            text,
            offset,
        }
    }

    fn rparen(&mut self) -> Token<'a> {
        debug_assert!(self.la == Some(')'));
        let offset = self.offset;
        let text = &self.text[offset..offset + 1];
        debug_assert!(text == ")");
        self.consume();
        Token {
            ty: TokenType::RParen,
            text,
            offset,
        }
    }

    fn lbrace(&mut self) -> Token<'a> {
        debug_assert!(self.la == Some('{'));
        let offset = self.offset;
        let text = &self.text[offset..offset + 1];
        debug_assert!(text == "{");
        self.consume();
        Token {
            ty: TokenType::LBrace,
            text,
            offset,
        }
    }

    fn rbrace(&mut self) -> Token<'a> {
        debug_assert!(self.la == Some('}'));
        let offset = self.offset;
        let text = &self.text[offset..offset + 1];
        debug_assert!(text == "}");
        self.consume();
        Token {
            ty: TokenType::RBrace,
            text,
            offset,
        }
    }

    fn lbrack(&mut self) -> Token<'a> {
        debug_assert!(self.la == Some('['));
        let offset = self.offset;
        let text = &self.text[offset..offset + 1];
        debug_assert!(text == "[");
        self.consume();
        Token {
            ty: TokenType::LBrack,
            text,
            offset,
        }
    }

    fn rbrack(&mut self) -> Token<'a> {
        debug_assert!(self.la == Some(']'));
        let offset = self.offset;
        let text = &self.text[offset..offset + 1];
        debug_assert!(text == "]");
        self.consume();
        Token {
            ty: TokenType::RBrack,
            text,
            offset,
        }
    }

    fn consume_whitespace(&mut self) {
        debug_assert!(self.la.unwrap().is_ascii_whitespace());
        while let Some(la) = self.la {
            if la.is_ascii_whitespace() {
                self.consume();
            } else {
                break;
            }
        }
    }

    fn consume_comment(&mut self) -> Result<(), KError> {
        debug_assert!(self.la == Some('/'));

        self.consume();
        let la = self.la_or_err()?;

        match la {
            '/' => {
                // Line comment
                self.consume();
                while let Some(la) = self.la {
                    self.consume();
                    if la == '\n' {
                        return Ok(());
                    }
                }
                Ok(())
            }

            '*' => {
                // Block comment
                self.consume();
                let mut la = self.la_or_err()?;
                loop {
                    while la != '*' {
                        self.consume();
                        la = self.la_or_err()?;
                    }
                    self.consume();
                    la = self.la_or_err()?;
                    if la == '/' {
                        self.consume();
                        return Ok(());
                    }
                }
            }

            _ => Err(KoreLexerError(format!(
                "Expected '/' or '*', got: {:?}",
                la
            ))),
        }
    }

    fn consume(&mut self) -> Option<char> {
        let res = self.la;
        self.la = self.it.next();
        if self.la.is_some() {
            self.offset += 1;
        }
        res
    }

    fn la_or_err(&self) -> Result<char, KError> {
        self.la
            .ok_or_else(|| KoreLexerError(String::from("Unexpected end of file")))
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        match self.next_token() {
            Ok(token) => match token.ty {
                TokenType::Eof => None,
                _ => Some(token),
            },
            Err(err) => panic!("{}", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Lexer, Token, TokenType};

    macro_rules! lexer_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                // Given
                let (text, expected) = $value;
                let lexer = Lexer::new(text);

                // When
                let actual: Vec<_> = lexer.into_iter().collect();

                // Then
                assert_eq!(expected, actual);
            }
        )*
        }
    }

    lexer_tests! {
        test_lexer_00: ("", Vec::<Token>::new()),
        test_lexer_01: (" ", Vec::<Token>::new()),
        test_lexer_02: ("\n", Vec::<Token>::new()),
        test_lexer_03: ("\r", Vec::<Token>::new()),
        test_lexer_04: ("\t", Vec::<Token>::new()),
        test_lexer_05: ("\x0c", Vec::<Token>::new()),
        test_lexer_06: ("//", Vec::<Token>::new()),
        test_lexer_07: ("/* foo */", Vec::<Token>::new()),
        test_lexer_08: (",", vec![Token { ty: TokenType::Comma, text: ",", offset: 0 }]),
        test_lexer_09: (":", vec![Token { ty: TokenType::Colon, text: ":", offset: 0 }]),
        test_lexer_10: (":=", vec![Token { ty: TokenType::Walrus, text: ":=", offset: 0 }]),
        test_lexer_11: ("(", vec![Token { ty: TokenType::LParen, text: "(", offset: 0 }]),
        test_lexer_12: (")", vec![Token { ty: TokenType::RParen, text: ")", offset: 0 }]),
        test_lexer_13: ("{", vec![Token { ty: TokenType::LBrace, text: "{", offset: 0 }]),
        test_lexer_14: ("}", vec![Token { ty: TokenType::RBrace, text: "}", offset: 0 }]),
        test_lexer_15: ("[", vec![Token { ty: TokenType::LBrack, text: "[", offset: 0 }]),
        test_lexer_16: ("]", vec![Token { ty: TokenType::RBrack, text: "]", offset: 0 }]),
        test_lexer_17: (r#""foo\"bar\\baz""#, vec![Token { ty: TokenType::Str, text: r#""foo\"bar\\baz""#, offset: 0 }]),
        test_lexer_18: ("foo", vec![Token { ty: TokenType::Id, text: r"foo", offset: 0 }]),
        test_lexer_19: (r"\foo", vec![Token { ty: TokenType::SymbolId, text: r"\foo", offset: 0 }]),
        test_lexer_20: ("@foo", vec![Token { ty: TokenType::SetVarId, text: "@foo", offset: 0 }]),
        test_lexer_21: (r"\top", vec![Token { ty: TokenType::MlTop, text: r"\top", offset: 0 }]),
        test_lexer_22: (r"\bottom", vec![Token { ty: TokenType::MlBottom, text: r"\bottom", offset: 0 }]),
        test_lexer_23: (r"\and", vec![Token { ty: TokenType::MlAnd, text: r"\and", offset: 0 }]),
        test_lexer_24: (r"\or", vec![Token { ty: TokenType::MlOr, text: r"\or", offset: 0 }]),
        test_lexer_25: (r"\implies", vec![Token { ty: TokenType::MlImplies, text: r"\implies", offset: 0 }]),
        test_lexer_26: (r"\iff", vec![Token { ty: TokenType::MlIff, text: r"\iff", offset: 0 }]),
        test_lexer_27: (r"\exists", vec![Token { ty: TokenType::MlExists, text: r"\exists", offset: 0 }]),
        test_lexer_28: (r"\forall", vec![Token { ty: TokenType::MlForall, text: r"\forall", offset: 0 }]),
        test_lexer_29: (r"\mu", vec![Token { ty: TokenType::MlMu, text: r"\mu", offset: 0 }]),
        test_lexer_30: (r"\nu", vec![Token { ty: TokenType::MlNu, text: r"\nu", offset: 0 }]),
        test_lexer_31: (r"\ceil", vec![Token { ty: TokenType::MlCeil, text: r"\ceil", offset: 0 }]),
        test_lexer_32: (r"\floor", vec![Token { ty: TokenType::MlFloor, text: r"\floor", offset: 0 }]),
        test_lexer_33: (r"\equals", vec![Token { ty: TokenType::MlEquals, text: r"\equals", offset: 0 }]),
        test_lexer_34: (r"\in", vec![Token { ty: TokenType::MlIn, text: r"\in", offset: 0 }]),
        test_lexer_35: (r"\next", vec![Token { ty: TokenType::MlNext, text: r"\next", offset: 0 }]),
        test_lexer_36: (r"\rewrites", vec![Token { ty: TokenType::MlRewrites, text: r"\rewrites", offset: 0 }]),
        test_lexer_37: (r"\dv", vec![Token { ty: TokenType::MlDv, text: r"\dv", offset: 0 }]),
        test_lexer_38: (r"\left-assoc", vec![Token { ty: TokenType::MlLeftAssoc, text: r"\left-assoc", offset: 0 }]),
        test_lexer_39: (r"\right-assoc", vec![Token { ty: TokenType::MlRightAssoc, text: r"\right-assoc", offset: 0 }]),
        test_lexer_40: (r"module", vec![Token { ty: TokenType::KwModule, text: r"module", offset: 0 }]),
        test_lexer_41: (r"endmodule", vec![Token { ty: TokenType::KwEndmodule, text: r"endmodule", offset: 0 }]),
        test_lexer_42: (r"import", vec![Token { ty: TokenType::KwImport, text: r"import", offset: 0 }]),
        test_lexer_43: (r"sort", vec![Token { ty: TokenType::KwSort, text: r"sort", offset: 0 }]),
        test_lexer_44: (r"hooked-sort", vec![Token { ty: TokenType::KwHookedSort, text: r"hooked-sort", offset: 0 }]),
        test_lexer_45: (r"symbol", vec![Token { ty: TokenType::KwSymbol, text: r"symbol", offset: 0 }]),
        test_lexer_46: (r"hooked-symbol", vec![Token { ty: TokenType::KwHookedSymbol, text: r"hooked-symbol", offset: 0 }]),
        test_lexer_47: (r"axiom", vec![Token { ty: TokenType::KwAxiom, text: r"axiom", offset: 0 }]),
        test_lexer_48: (r"claim", vec![Token { ty: TokenType::KwClaim, text: r"claim", offset: 0 }]),
        test_lexer_49: (r"alias", vec![Token { ty: TokenType::KwAlias, text: r"alias", offset: 0 }]),
        test_lexer_50: (r"where", vec![Token { ty: TokenType::KwWhere, text: r"where", offset: 0 }]),
        test_lexer_51: (
            r#"\dv{SortInt{}}("0")"#,
            vec![
                Token { ty: TokenType::MlDv, text: r"\dv", offset: 0 },
                Token { ty: TokenType::LBrace, text: "{", offset: 3 },
                Token { ty: TokenType::Id, text: "SortInt", offset: 4 },
                Token { ty: TokenType::LBrace, text: "{", offset: 11 },
                Token { ty: TokenType::RBrace, text: "}", offset: 12 },
                Token { ty: TokenType::RBrace, text: "}", offset: 13 },
                Token { ty: TokenType::LParen, text: "(", offset: 14 },
                Token { ty: TokenType::Str, text: r#""0""#, offset: 15 },
                Token { ty: TokenType::RParen, text: ")", offset: 18 },
            ],
        ),
    }
}

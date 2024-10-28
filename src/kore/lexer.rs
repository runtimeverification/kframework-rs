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

    pub fn offset(&self) -> usize {
        self.offset
    }
}

#[derive(Debug)]
pub struct KoreLexer<'a> {
    text: &'a str,
    it: Chars<'a>,
    la: Option<char>,
    offset: usize,
}

impl<'a> KoreLexer<'a> {
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

    pub fn next_token(&mut self) -> Result<Token<'a>, String> {
        // TODO KoreLexerError
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
            _ => return Err(format!("Unexpected character: {:?}", la)),
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

    fn string(&mut self) -> Result<Token<'a>, String> {
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

    fn symbol_or_ml_conn(&mut self) -> Result<Token<'a>, String> {
        debug_assert!(self.la == Some('\\'));

        let offset = self.offset;
        let mut len: usize = 2;

        self.consume();
        let la = self.la_or_err()?;
        if !la.is_ascii_alphabetic() {
            return Err(format!("Expected letter, got: {:?}", la));
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

    fn set_var_id(&mut self) -> Result<Token<'a>, String> {
        debug_assert!(self.la == Some('@'));

        let offset = self.offset;
        let mut len: usize = 2;

        // TODO consume_alphabetic
        self.consume();
        let la = self.la_or_err()?;
        if !la.is_ascii_alphabetic() {
            return Err(format!("Expected letter, got: {:?}", la));
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

    fn consume_comment(&mut self) -> Result<(), String> {
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

            _ => Err(format!("Expected '/' or '*', got: {:?}", la)),
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

    fn la_or_err(&self) -> Result<char, String> {
        self.la
            .ok_or_else(|| String::from("Unexpected end of file"))
    }
}

impl<'a> Iterator for KoreLexer<'a> {
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

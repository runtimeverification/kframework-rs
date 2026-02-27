use super::lexer::{Lexer, Token, TokenType};
use super::{
    App, Definition, Id, Module, Pattern, SVar, Sentence, SetVarId, Sort, Str, SymbolId, Var,
};

pub struct Parser<'a> {
    #[allow(dead_code)]
    text: &'a str,
    lexer: Lexer<'a>,
    la: Token<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Result<Self, String> {
        // TODO KoreParserError
        let mut lexer = Lexer::new(text);
        let la = lexer.next_token()?;
        Ok(Self { text, lexer, la })
    }

    pub fn definition(&mut self) -> Result<Definition, String> {
        let attrs = self.attrs()?;
        let mut modules = Vec::new();
        while self.la.ty() != TokenType::Eof {
            let module = self.module()?;
            modules.push(module);
        }
        Ok(Definition { modules, attrs })
    }

    pub fn module(&mut self) -> Result<Module, String> {
        self.match_token(TokenType::KwModule)?;
        let id = self.id()?;
        let mut sentences = Vec::new();
        while self.la.ty() != TokenType::KwEndmodule {
            let sentence = self.sentence()?;
            sentences.push(sentence);
        }
        self.consume()?;
        let attrs = self.attrs()?;
        Ok(Module {
            id,
            sentences,
            attrs,
        })
    }

    pub fn sentence(&mut self) -> Result<Sentence, String> {
        let parse = match self.la.ty() {
            TokenType::KwImport => Self::import,
            TokenType::KwSort => Self::ssort,
            TokenType::KwHookedSort => Self::hooked_sort,
            TokenType::KwSymbol => Self::symbol,
            TokenType::KwHookedSymbol => Self::hooked_symbol,
            TokenType::KwAlias => Self::alias,
            TokenType::KwAxiom => Self::axiom,
            TokenType::KwClaim => Self::claim,
            _ => {
                return Err(format!(
                    "Expected sentence token, found: {:?}",
                    self.la.ty()
                ))
            }
        };
        parse(self)
    }

    pub fn pattern(&mut self) -> Result<Pattern, String> {
        let parse = match self.la.ty() {
            TokenType::Str => Self::str,
            TokenType::SetVarId => Self::svar,
            TokenType::SymbolId => Self::app,
            TokenType::Id => Self::var_or_app,
            TokenType::MlLeftAssoc => Self::left_assoc,
            TokenType::MlRightAssoc => Self::right_assoc,
            TokenType::MlDv => Self::dv,
            TokenType::MlTop => Self::top,
            TokenType::MlBottom => Self::bottom,
            TokenType::MlNot => Self::not,
            TokenType::MlImplies => Self::implies,
            TokenType::MlIff => Self::iff,
            TokenType::MlAnd => Self::and,
            TokenType::MlOr => Self::or,
            TokenType::MlExists => Self::exists,
            TokenType::MlForall => Self::forall,
            TokenType::MlMu => Self::mu,
            TokenType::MlNu => Self::nu,
            TokenType::MlCeil => Self::ceil,
            TokenType::MlFloor => Self::floor,
            TokenType::MlEquals => Self::equals,
            TokenType::MlIn => Self::inn,
            TokenType::MlNext => Self::next,
            TokenType::MlRewrites => Self::rewrites,
            _ => return Err(format!("Expected pattern token, found: {:?}", self.la.ty())),
        };
        parse(self)
    }

    pub fn sort(&mut self) -> Result<Sort, String> {
        let id = self.id()?;
        let sort = if self.la.ty() == TokenType::LBrace {
            let args = self.sorts()?;
            Sort::App { id, args }
        } else {
            Sort::Var(id)
        };
        Ok(sort)
    }

    /*
     * Helpers: sentences
     */

    fn import(&mut self) -> Result<Sentence, String> {
        debug_assert!(self.la.ty() == TokenType::KwImport);
        self.consume()?;
        let module = self.id()?;
        let attrs = self.attrs()?;
        Ok(Sentence::Import { module, attrs })
    }

    fn ssort(&mut self) -> Result<Sentence, String> {
        debug_assert!(self.la.ty() == TokenType::KwSort);
        self.consume()?;
        let id = self.id()?;
        let vars = self.sort_vars()?;
        let attrs = self.attrs()?;
        Ok(Sentence::Sort {
            id,
            vars,
            attrs,
            hooked: false,
        })
    }

    fn hooked_sort(&mut self) -> Result<Sentence, String> {
        debug_assert!(self.la.ty() == TokenType::KwHookedSort);
        self.consume()?;
        let id = self.id()?;
        let vars = self.sort_vars()?;
        let attrs = self.attrs()?;
        Ok(Sentence::Sort {
            id,
            vars,
            attrs,
            hooked: true,
        })
    }

    fn symbol(&mut self) -> Result<Sentence, String> {
        debug_assert!(self.la.ty() == TokenType::KwSymbol);
        self.consume()?;
        let id = self.symbol_id()?;
        let vars = self.sort_vars()?;
        let param_sorts = self.param_sorts()?;
        self.match_token(TokenType::Colon)?;
        let sort = self.sort()?;
        let attrs = self.attrs()?;
        Ok(Sentence::Symbol {
            id,
            vars,
            param_sorts,
            sort,
            attrs,
            hooked: false,
        })
    }

    fn hooked_symbol(&mut self) -> Result<Sentence, String> {
        debug_assert!(self.la.ty() == TokenType::KwHookedSymbol);
        self.consume()?;
        let id = self.symbol_id()?;
        let vars = self.sort_vars()?;
        let param_sorts = self.param_sorts()?;
        self.match_token(TokenType::Colon)?;
        let sort = self.sort()?;
        let attrs = self.attrs()?;
        Ok(Sentence::Symbol {
            id,
            vars,
            param_sorts,
            sort,
            attrs,
            hooked: true,
        })
    }

    fn alias(&mut self) -> Result<Sentence, String> {
        debug_assert!(self.la.ty() == TokenType::KwAlias);
        self.consume()?;
        let id = self.symbol_id()?;
        let vars = self.sort_vars()?;
        let param_sorts = self.param_sorts()?;
        self.match_token(TokenType::Colon)?;
        let sort = self.sort()?;
        self.match_token(TokenType::KwWhere)?;
        let left = self.app_()?;
        self.match_token(TokenType::Walrus)?;
        let right = self.pattern()?;
        let attrs = self.attrs()?;
        Ok(Sentence::Alias {
            id,
            vars,
            param_sorts,
            sort,
            left,
            right: Box::new(right),
            attrs,
        })
    }

    fn axiom(&mut self) -> Result<Sentence, String> {
        debug_assert!(self.la.ty() == TokenType::KwAxiom);
        self.consume()?;
        let vars = self.sort_vars()?;
        let pattern = self.pattern()?;
        let attrs = self.attrs()?;
        Ok(Sentence::Axiom {
            vars,
            pattern: Box::new(pattern),
            attrs,
        })
    }

    fn claim(&mut self) -> Result<Sentence, String> {
        debug_assert!(self.la.ty() == TokenType::KwClaim);
        self.consume()?;
        let vars = self.sort_vars()?;
        let pattern = self.pattern()?;
        let attrs = self.attrs()?;
        Ok(Sentence::Claim {
            vars,
            pattern: Box::new(pattern),
            attrs,
        })
    }

    /*
     * Helpers: patterns
     */

    fn str(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::Str);
        let s = self.str_()?;
        Ok(Pattern::Str(s))
    }

    fn svar(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::SetVarId);
        let svar = self.svar_()?;
        Ok(Pattern::SVar(svar))
    }

    fn var_or_app(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::Id);
        let id = self.id()?;
        if self.la.ty() == TokenType::Colon {
            // Var
            self.consume()?;
            let sort = self.sort()?;
            let var = Var { id, sort };
            Ok(Pattern::Var(var))
        } else {
            // App
            let symbol = SymbolId(id.0);
            let sorts = self.sorts()?;
            let args = self.patterns()?;
            Ok(Pattern::App(App {
                symbol,
                sorts,
                args,
            }))
        }
    }

    fn app(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::SymbolId);
        let app = self.app_()?;
        Ok(Pattern::App(app))
    }

    // TODO macros

    fn left_assoc(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlLeftAssoc);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        self.match_token(TokenType::RBrace)?;
        self.match_token(TokenType::LParen)?;
        let app = self.app_()?;
        self.match_token(TokenType::RParen)?;
        Ok(Pattern::LeftAssoc(app))
    }

    fn right_assoc(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlRightAssoc);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        self.match_token(TokenType::RBrace)?;
        self.match_token(TokenType::LParen)?;
        let app = self.app_()?;
        self.match_token(TokenType::RParen)?;
        Ok(Pattern::RightAssoc(app))
    }

    fn dv(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlDv);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        let sort = self.sort()?;
        self.match_token(TokenType::RBrace)?;
        self.match_token(TokenType::LParen)?;
        let value = self.str_()?;
        self.match_token(TokenType::RParen)?;
        Ok(Pattern::Dv { sort, value })
    }

    fn top(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlTop);
        self.consume()?;
        // TODO extract helpers for "{" Sort "}" etc.
        self.match_token(TokenType::LBrace)?;
        let sort = self.sort()?;
        self.match_token(TokenType::RBrace)?;
        self.match_token(TokenType::LParen)?;
        self.match_token(TokenType::RParen)?;
        Ok(Pattern::Top(sort))
    }

    fn bottom(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlBottom);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        let sort = self.sort()?;
        self.match_token(TokenType::RBrace)?;
        self.match_token(TokenType::LParen)?;
        self.match_token(TokenType::RParen)?;
        Ok(Pattern::Bottom(sort))
    }

    fn not(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlNot);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        let sort = self.sort()?;
        self.match_token(TokenType::RBrace)?;
        self.match_token(TokenType::LParen)?;
        let op = self.pattern()?;
        self.match_token(TokenType::RParen)?;
        Ok(Pattern::Not {
            sort,
            op: Box::new(op),
        })
    }

    fn implies(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlImplies);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        let sort = self.sort()?;
        self.match_token(TokenType::RBrace)?;
        self.match_token(TokenType::LParen)?;
        let left = self.pattern()?;
        self.match_token(TokenType::Comma)?;
        let right = self.pattern()?;
        self.match_token(TokenType::RParen)?;
        Ok(Pattern::Implies {
            sort,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn iff(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlIff);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        let sort = self.sort()?;
        self.match_token(TokenType::RBrace)?;
        self.match_token(TokenType::LParen)?;
        let left = self.pattern()?;
        self.match_token(TokenType::Comma)?;
        let right = self.pattern()?;
        self.match_token(TokenType::RParen)?;
        Ok(Pattern::Iff {
            sort,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn and(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlAnd);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        let sort = self.sort()?;
        self.match_token(TokenType::RBrace)?;
        let ops = self.patterns()?;
        Ok(Pattern::And { sort, ops })
    }

    fn or(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlOr);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        let sort = self.sort()?;
        self.match_token(TokenType::RBrace)?;
        let ops = self.patterns()?;
        Ok(Pattern::Or { sort, ops })
    }

    fn exists(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlExists);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        let sort = self.sort()?;
        self.match_token(TokenType::RBrace)?;
        self.match_token(TokenType::LParen)?;
        let var = self.var()?;
        self.match_token(TokenType::Comma)?;
        let op = self.pattern()?;
        self.match_token(TokenType::RParen)?;
        Ok(Pattern::Exists {
            sort,
            var,
            op: Box::new(op),
        })
    }

    fn forall(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlForall);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        let sort = self.sort()?;
        self.match_token(TokenType::RBrace)?;
        self.match_token(TokenType::LParen)?;
        let var = self.var()?;
        self.match_token(TokenType::Comma)?;
        let op = self.pattern()?;
        self.match_token(TokenType::RParen)?;
        Ok(Pattern::Forall {
            sort,
            var,
            op: Box::new(op),
        })
    }

    fn mu(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlMu);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        self.match_token(TokenType::RBrace)?;
        self.match_token(TokenType::LParen)?;
        let var = self.svar_()?;
        self.match_token(TokenType::Comma)?;
        let op = self.pattern()?;
        self.match_token(TokenType::RParen)?;
        Ok(Pattern::Mu {
            var,
            op: Box::new(op),
        })
    }

    fn nu(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlNu);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        self.match_token(TokenType::RBrace)?;
        self.match_token(TokenType::LParen)?;
        let var = self.svar_()?;
        self.match_token(TokenType::Comma)?;
        let op = self.pattern()?;
        self.match_token(TokenType::RParen)?;
        Ok(Pattern::Nu {
            var,
            op: Box::new(op),
        })
    }

    fn ceil(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlCeil);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        let op_sort = self.sort()?;
        self.match_token(TokenType::Comma)?;
        let sort = self.sort()?;
        self.match_token(TokenType::RBrace)?;
        self.match_token(TokenType::LParen)?;
        let op = self.pattern()?;
        self.match_token(TokenType::RParen)?;
        Ok(Pattern::Ceil {
            op_sort,
            sort,
            op: Box::new(op),
        })
    }

    fn floor(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlFloor);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        let op_sort = self.sort()?;
        self.match_token(TokenType::Comma)?;
        let sort = self.sort()?;
        self.match_token(TokenType::RBrace)?;
        self.match_token(TokenType::LParen)?;
        let op = self.pattern()?;
        self.match_token(TokenType::RParen)?;
        Ok(Pattern::Floor {
            op_sort,
            sort,
            op: Box::new(op),
        })
    }

    fn equals(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlEquals);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        let op_sort = self.sort()?;
        self.match_token(TokenType::Comma)?;
        let sort = self.sort()?;
        self.match_token(TokenType::RBrace)?;
        self.match_token(TokenType::LParen)?;
        let left = self.pattern()?;
        self.match_token(TokenType::Comma)?;
        let right = self.pattern()?;
        self.match_token(TokenType::RParen)?;
        Ok(Pattern::Equals {
            op_sort,
            sort,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn inn(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlIn);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        let op_sort = self.sort()?;
        self.match_token(TokenType::Comma)?;
        let sort = self.sort()?;
        self.match_token(TokenType::RBrace)?;
        self.match_token(TokenType::LParen)?;
        let left = self.pattern()?;
        self.match_token(TokenType::Comma)?;
        let right = self.pattern()?;
        self.match_token(TokenType::RParen)?;
        Ok(Pattern::In {
            op_sort,
            sort,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn next(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlNext);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        let sort = self.sort()?;
        self.match_token(TokenType::RBrace)?;
        self.match_token(TokenType::LParen)?;
        let op = self.pattern()?;
        self.match_token(TokenType::RParen)?;
        Ok(Pattern::Next {
            sort,
            op: Box::new(op),
        })
    }

    fn rewrites(&mut self) -> Result<Pattern, String> {
        debug_assert!(self.la.ty() == TokenType::MlRewrites);
        self.consume()?;
        self.match_token(TokenType::LBrace)?;
        let sort = self.sort()?;
        self.match_token(TokenType::RBrace)?;
        self.match_token(TokenType::LParen)?;
        let left = self.pattern()?;
        self.match_token(TokenType::Comma)?;
        let right = self.pattern()?;
        self.match_token(TokenType::RParen)?;
        Ok(Pattern::Rewrites {
            sort,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    /*
     * Helpers: misc
     */

    fn id(&mut self) -> Result<Id, String> {
        let s = self.match_token(TokenType::Id)?;
        Ok(Id(String::from(s)))
    }

    fn symbol_id(&mut self) -> Result<SymbolId, String> {
        match self.la.ty() {
            TokenType::Id | TokenType::SymbolId => {
                let s = String::from(self.la.text());
                self.consume()?;
                Ok(SymbolId(s))
            }
            _ => Err(format!(
                "Expected token {:?} or {:?}, found: {:?}",
                TokenType::Id,
                TokenType::SymbolId,
                self.la.ty()
            )),
        }
    }

    fn set_var_id(&mut self) -> Result<SetVarId, String> {
        let s = self.match_token(TokenType::SetVarId)?;
        Ok(SetVarId(String::from(s)))
    }

    fn str_(&mut self) -> Result<Str, String> {
        let s = self.match_token(TokenType::Str)?;
        Str::from_kore(&s[1..s.len() - 1])
    }

    fn var(&mut self) -> Result<Var, String> {
        let id = self.id()?;
        self.match_token(TokenType::Colon)?;
        let sort = self.sort()?;
        Ok(Var { id, sort })
    }

    fn svar_(&mut self) -> Result<SVar, String> {
        let id = self.set_var_id()?;
        self.match_token(TokenType::Colon)?;
        let sort = self.sort()?;
        Ok(SVar { id, sort })
    }

    fn app_(&mut self) -> Result<App, String> {
        let symbol = self.symbol_id()?;
        let sorts = self.sorts()?;
        let args = self.patterns()?;
        Ok(App {
            symbol,
            sorts,
            args,
        })
    }

    /*
     * Helpers: delimited lists
     */

    fn sort_vars(&mut self) -> Result<Vec<Id>, String> {
        self.delimited_list(
            Self::id,
            TokenType::LBrace,
            TokenType::RBrace,
            TokenType::Comma,
        )
    }

    fn attrs(&mut self) -> Result<Vec<App>, String> {
        self.delimited_list(
            Self::app_,
            TokenType::LBrack,
            TokenType::RBrack,
            TokenType::Comma,
        )
    }

    fn patterns(&mut self) -> Result<Vec<Pattern>, String> {
        self.delimited_list(
            Self::pattern,
            TokenType::LParen,
            TokenType::RParen,
            TokenType::Comma,
        )
    }

    fn sorts(&mut self) -> Result<Vec<Sort>, String> {
        self.delimited_list(
            Self::sort,
            TokenType::LBrace,
            TokenType::RBrace,
            TokenType::Comma,
        )
    }

    fn param_sorts(&mut self) -> Result<Vec<Sort>, String> {
        self.delimited_list(
            Self::sort,
            TokenType::LParen,
            TokenType::RParen,
            TokenType::Comma,
        )
    }

    fn delimited_list<T>(
        &mut self,
        parse: fn(&mut Self) -> Result<T, String>,
        ldelim: TokenType,
        rdelim: TokenType,
        sep: TokenType,
    ) -> Result<Vec<T>, String> {
        self.match_token(ldelim)?;
        let mut elems = Vec::new();
        while self.la.ty() != rdelim {
            let elem = parse(self)?;
            elems.push(elem);
            if self.la.ty() != sep {
                break;
            }
            self.consume()?;
        }
        self.consume()?;
        Ok(elems)
    }

    /*
     * Helpers: lexer management
     */

    fn match_token(&mut self, expected: TokenType) -> Result<&'a str, String> {
        let actual = self.la.ty();
        if actual == expected {
            let text = self.la.text();
            self.consume()?;
            Ok(text)
        } else {
            Err(format!(
                "Expected token {:?}, found: {:?}",
                expected, actual,
            ))
        }
    }

    fn consume(&mut self) -> Result<(), String> {
        self.la = self.lexer.next_token()?;
        Ok(())
    }
}

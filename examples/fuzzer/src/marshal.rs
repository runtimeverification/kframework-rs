#![allow(unused)]

use kframework::kore::{App, Pattern, Sort, SymbolId};
use kframework_ffi::kllvm;
use kframework_ffi::kllvm::ffi;
use std::collections::HashMap;
use std::ffi::CString;

#[derive(Debug)]
pub enum MarshalError {
    Unsupported(&'static str),
    UnknownVar(String),
    Cstring,
}

pub trait VarHandler {
    fn substitute(&mut self, name: &str) -> Result<Pattern, MarshalError>;
}

struct OwnedSymbol(*mut ffi::kore_symbol);
impl Drop for OwnedSymbol {
    fn drop(&mut self) {
        unsafe { ffi::kore_symbol_free(self.0) }
    }
}

struct OwnedPattern(*mut ffi::kore_pattern);
impl Drop for OwnedPattern {
    fn drop(&mut self) {
        unsafe { ffi::kore_pattern_free(self.0) }
    }
}

struct OwnedSort(*mut ffi::kore_sort);
impl Drop for OwnedSort {
    fn drop(&mut self) {
        unsafe { ffi::kore_sort_free(self.0) }
    }
}

pub struct Marshaller<H: VarHandler> {
    symbols: HashMap<String, OwnedSymbol>,
    subtrees: HashMap<*const Pattern, OwnedPattern>,
    handler: Option<H>,
}

impl<H: VarHandler> Marshaller<H> {
    pub fn new(handler: Option<H>) -> Self {
        Self {
            symbols: HashMap::new(),
            subtrees: HashMap::new(),
            handler,
        }
    }

    pub fn marshal(&mut self, root: &Pattern) -> Result<kllvm::Pattern, MarshalError> {
        let (raw, _var_free, fresh) = self.marshal_node(root, false)?;
        // If the root happened to be fully var-free it now lives in the
        // cache; transfer ownership out so kllvm::Pattern's Drop is the
        // sole freer. mem::forget the OwnedPattern we popped — the cache
        // no longer owns it; kllvm::Pattern does.
        if !fresh {
            if let Some(owned) = self.subtrees.remove(&(root as *const Pattern)) {
                std::mem::forget(owned);
            }
        }
        Ok(kllvm::Pattern::from_raw(raw))
    }

    pub fn set_handler(&mut self, h: H) {
        self.handler = Some(h);
    }

    /// Returns `(ptr, var_free, fresh)`.
    /// - `fresh = true`  => caller owns the C wrapper and must free it
    ///   (typically right after handing it to `add_argument`, since the
    ///   parent already took a `shared_ptr` copy of the underlying AST).
    /// - `fresh = false` => `ptr` is borrowed (lives in `self.subtrees`).
    fn marshal_node(
        &mut self,
        p: &Pattern,
        was_var: bool,
    ) -> Result<(*mut ffi::kore_pattern, bool, bool), MarshalError> {
        if let Some(owned) = self.subtrees.get(&(p as *const Pattern)) {
            return Ok((owned.0, true, false));
        }

        let res = match p {
            Pattern::Var(v) => {
                let handler = self
                    .handler
                    .as_mut()
                    .ok_or_else(|| MarshalError::UnknownVar(v.id.as_str().into()))?;
                let sub = handler.substitute(v.id.as_str())?;
                let (ptr, _, fresh) = self.marshal_node(&sub, true)?;
                Ok((ptr, false, fresh))
            }

            Pattern::Dv { sort, value } => {
                let sort_owned = build_sort(sort)?;
                let c_val = CString::new(value.0.as_str()).map_err(|_| MarshalError::Cstring)?;
                let ptr = unsafe { ffi::kore_pattern_new_token(c_val.as_ptr(), sort_owned.0) };
                Ok((ptr, true, true))
            }

            Pattern::App(app) => self.marshal_app(app),

            other => Err(MarshalError::Unsupported(variant_name(other))),
        };

        let (ptr, var_free, mut fresh) = res?;
        if !was_var && var_free && fresh {
            self.subtrees
                .insert(p as *const Pattern, OwnedPattern(ptr));
            fresh = false;
        }
        Ok((ptr, var_free, fresh))
    }

    fn marshal_app(
        &mut self,
        app: &App,
    ) -> Result<(*mut ffi::kore_pattern, bool, bool), MarshalError> {
        let sym = self.intern_symbol(&app.symbol, &app.sorts)?;
        let pat = unsafe { ffi::kore_composite_pattern_from_symbol(sym) };

        let mut var_free = true;
        for arg in &app.args {
            let (child, child_vf, child_fresh) = self.marshal_node(arg, false)?;
            unsafe { ffi::kore_composite_pattern_add_argument(pat, child) };
            // add_argument copied the inner shared_ptr; the underlying AST
            // is now retained by `pat`. A fresh child wrapper is no longer
            // needed and must be freed here, otherwise every var-bearing
            // intermediate node leaks its C wrapper struct each iteration
            // (which was the dominant slow drip in the earlier MVP).
            // Borrowed (cached) children must NOT be freed.
            if child_fresh {
                unsafe { ffi::kore_pattern_free(child) };
            }
            var_free &= child_vf;
        }

        // `pat` is freshly allocated in this call. Caller (or
        // marshal_node's caching step) decides what to do with it.
        Ok((pat, var_free, true))
    }

    fn intern_symbol(
        &mut self,
        id: &SymbolId,
        sorts: &[Sort],
    ) -> Result<*mut ffi::kore_symbol, MarshalError> {
        let key = id.as_str().to_owned();
        if let Some(owned) = self.symbols.get(&key) {
            return Ok(owned.0);
        }
        let c_name = CString::new(id.as_str()).map_err(|_| MarshalError::Cstring)?;
        let sym = unsafe { ffi::kore_symbol_new(c_name.as_ptr()) };
        for sort in sorts {
            let s = build_sort(sort)?;
            unsafe { ffi::kore_symbol_add_formal_argument(sym, s.0) };
            // s drops -> kore_sort_free; the symbol kept a shared_ptr copy.
        }
        self.symbols.insert(key, OwnedSymbol(sym));
        Ok(sym)
    }
}

fn build_sort(s: &Sort) -> Result<OwnedSort, MarshalError> {
    match s {
        Sort::App { id, args } => {
            let c_name = CString::new(id.as_str()).map_err(|_| MarshalError::Cstring)?;
            let raw = unsafe { ffi::kore_composite_sort_new(c_name.as_ptr()) };
            for arg in args {
                let arg_sort = build_sort(arg)?;
                unsafe { ffi::kore_composite_sort_add_argument(raw, arg_sort.0) };
            }
            Ok(OwnedSort(raw))
        }
        Sort::Var(_) => Err(MarshalError::Unsupported("Sort::Var")),
    }
}

fn variant_name(p: &Pattern) -> &'static str {
    match p {
        Pattern::Var(_) => "Var",
        Pattern::SVar(_) => "SVar",
        Pattern::Str(_) => "Str",
        Pattern::App(_) => "App",
        Pattern::LeftAssoc(_) => "LeftAssoc",
        Pattern::RightAssoc(_) => "RightAssoc",
        Pattern::Top(_) => "Top",
        Pattern::Bottom(_) => "Bottom",
        Pattern::Dv { .. } => "Dv",
        Pattern::Not { .. } => "Not",
        Pattern::Implies { .. } => "Implies",
        Pattern::Iff { .. } => "Iff",
        Pattern::And { .. } => "And",
        Pattern::Or { .. } => "Or",
        Pattern::Exists { .. } => "Exists",
        Pattern::Forall { .. } => "Forall",
        Pattern::Mu { .. } => "Mu",
        Pattern::Nu { .. } => "Nu",
        Pattern::Ceil { .. } => "Ceil",
        Pattern::Floor { .. } => "Floor",
        Pattern::Equals { .. } => "Equals",
        Pattern::In { .. } => "In",
        Pattern::Next { .. } => "Next",
        Pattern::Rewrites { .. } => "Rewrites",
    }
}

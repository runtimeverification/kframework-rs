import pytest

from kframework_py.kore.syntax import Sort, SortVar, SortApp
from kframework_py.kore.syntax import (
    Pattern, EVar, SVar, String, App, LeftAssoc, RightAssoc,
    Top, Bottom, DV, Not, Implies, Iff, And, Or,
    Exists, Forall, Mu, Nu, Ceil, Floor, Equals, In, Next, Rewrites,
)
from kframework_py.kore.syntax import (
    Sentence, Symbol, Import, SortDecl, SymbolDecl, AliasDecl, Axiom, Claim,
)
from kframework_py.kore.syntax import Module, Definition

sort_int, sort_kitem = [SortApp(s, ()) for s in ["SortInt", "SortKItem"]]
sortvar_int, sortvar_kitem = [SortVar(s) for s in ["SortInt", "SortKItem"]]
one, two, three = [DV(sort_int, s) for s in ["1", "2", "3"]]
v1, v2, v3 = [EVar(vstr, sort_kitem) for vstr in ["V1", "V2", "V3"]]
sv1, sv2, sv3 = [SVar(svstr, sort_kitem) for svstr in ["@V1", "@V2", "@V3"]]
app1 = App("Sym1", (sort_kitem, sort_kitem), (one, two))
sym = Symbol("\\dv", (sortvar_int,))

def test_symbol():
    sym = Symbol("\\dv", (sortvar_int,))

    with pytest.raises(TypeError):
        Symbol("\\dv", sortvar_int)

    with pytest.raises(ValueError):
        Symbol("@V", (sortvar_int,))

def test_import():
    import_ = Import("MODNAME", (app1,))

    with pytest.raises(TypeError):
        Import("MODNAME", (sort_int,))

    with pytest.raises(ValueError):
        Import("@Setvar", (app1,))

def test_sortdecl():
    sortdecl = SortDecl("Sort", (sortvar_int, sortvar_int))
    sortdecl2 = SortDecl("Sort", ("Var1", "Var2"))

    with pytest.raises(TypeError):
        SortDecl("Sort", (sv1, sv2))

    with pytest.raises(ValueError):
        SortDecl("@SVar", ())

def test_symboldecl():
    symboldecl = SymbolDecl(sym, (sortvar_int, sortvar_int), sort_kitem)

    with pytest.raises(TypeError):
        SymbolDecl(sym, ("Var1", "Var2"), sort_kitem)

def test_aliasdecl():
    alias = AliasDecl(sym, (sortvar_int,), sort_kitem, app1, app1)

    with pytest.raises(TypeError):
        AliasDecl(sym, ("Sort",), sort_kitem, app1, app1)

def test_axiom():
    axiom = Axiom((sortvar_int,), app1)
    axiom = Axiom(("SortInt",), app1)

    with pytest.raises(TypeError):
        Axiom((sortvar_int,), sort_int)

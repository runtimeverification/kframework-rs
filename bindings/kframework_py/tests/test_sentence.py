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

def test_symbol():
    sym = Symbol("\\dv", (sortvar_int,))

    with pytest.raises(TypeError):
        Symbol("\\dv", sortvar_int)

    with pytest.raises(ValueError):
        Symbol("@V", (sortvar_int,))

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
one, two, three = [DV(sort_int, s) for s in ["1", "2", "3"]]

def test_app_let():
    app = App("LblProduction", (sort_int, sort_int, sort_int), (one, two, three))

    new_args = (three, two, one)
    new_app = app.let(args=new_args)

    new_args2 = [two, one, three]
    new_app2 = app.let_patterns(new_args2)

    assert new_app.args == new_args
    assert new_app2.args == tuple(new_args2)

    with pytest.raises(TypeError):
        bad_args = (1, 2, 3)
        app.let_patterns(bad_args)

def test_leftassoc_let():
    leftassoc = LeftAssoc("LblProduction", (sort_int, sort_int, sort_int), (one, two, three))

    new_args = (three, two, one)
    new_app = leftassoc.let(args=new_args)

    new_args2 = [two, one, three]
    new_app2 = leftassoc.let_patterns(new_args2)

    assert new_app.args == new_args
    assert new_app2.args == tuple(new_args2)

    with pytest.raises(TypeError):
        bad_args = (1, 2, 3)
        leftassoc.let_patterns(bad_args)

def test_rightassoc_let():
    rightassoc = RightAssoc("LblProduction", (sort_int, sort_int, sort_int), (one, two, three))

    new_args = (three, two, one)
    new_app = rightassoc.let(args=new_args)

    new_args2 = [two, one, three]
    new_app2 = rightassoc.let_patterns(new_args2)

    assert new_app.args == new_args
    assert new_app2.args == tuple(new_args2)

    with pytest.raises(TypeError):
        bad_args = (1, 2, 3)
        rightassoc.let_patterns(bad_args)

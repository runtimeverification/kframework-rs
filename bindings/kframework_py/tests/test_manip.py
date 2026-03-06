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

def test_sortvar_let():
    sortvar = SortVar("SortA")

    new_sort = sortvar.let(name="SortB")
    assert new_sort == SortVar("SortB")

    with pytest.raises(TypeError):
        sortvar.let(name=1)

def test_sortapp_let():
    sortapp = SortApp("SortApp", (sort_int, sort_kitem))

    new_sort = sortapp.let(sorts=(sort_kitem,))
    assert new_sort == SortApp("SortApp", (sort_kitem,))

    with pytest.raises(TypeError):
        sortapp.let(name=1)

def test_evar_let():
    evar = EVar("X", sort_kitem)

    new_evar = evar.let(sort=sort_int)
    new_evar2 = evar.let_patterns(())

    assert evar.patterns == ()
    assert new_evar == EVar("X", sort_int)
    assert new_evar2 is evar

    with pytest.raises(TypeError):
        evar.let(sort="SortBool")

def test_svar_let():
    svar = SVar("@Y", sort_kitem)

    new_svar = svar.let(sort=sort_int)
    new_svar2 = svar.let_patterns(())

    assert svar.patterns == ()
    assert new_svar == SVar("@Y", sort_int)
    assert new_svar2 is svar

    with pytest.raises(TypeError):
        svar.let(sort="SortApp")

def test_string_let():
    string = String("Kore String")

    new_string = string.let(value="A different kore string")
    new_string2 = string.let_patterns(())

    assert string.patterns == ()
    assert new_string == String("A different kore string")
    assert new_string2 is string

    with pytest.raises(TypeError):
        string.let(value=45)

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

def test_top_let():
    top = Top(sort_kitem)

    new_top = top.let(sort=sort_int)

    new_top2 = top.let_patterns(())

    assert top.patterns == ()
    assert new_top.sort == sort_int
    assert new_top2 is top

    with pytest.raises(TypeError):
        top.let(sort="SortInt")

def test_bottom_let():
    bottom = Bottom(sort_kitem)

    new_bottom = bottom.let(sort=sort_int)

    new_bottom2 = bottom.let_patterns(())

    assert bottom.patterns == ()
    assert new_bottom.sort == sort_int
    assert new_bottom2 is bottom

    with pytest.raises(TypeError):
        bottom.let(sort="SortInt")

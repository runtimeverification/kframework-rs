from kframework_py.kore.syntax import Id, Sort, SortVar, SortApp

def test_id():
    i = Id("foo")

    a = i.value
    b = i.value

    assert a == "foo"

    # TODO: Fix this!
    # assert a is b

def test_sort():
    # Parsing
    sort = Sort.parse("SortA{SortB{SortC}, SortD, SortE{SortF, SortG{}}}")

    # Getters and Constructors don't re-allocate
    arg0 = sort.args[0]
    arg1 = sort.args[1]
    reused = SortApp("Reused", (arg0, arg1))
    assert arg0 is sort.args[0]
    assert arg1 is reused.args[1]

    # Dataclass repr
    expected_repr = "SortApp(name='SortB', args=(SortVar(name='SortC'),))"
    assert arg0.__repr__() == expected_repr

    # Structural equivalence
    app = SortApp("SortB", (SortVar("SortC"),))
    assert arg0 == app

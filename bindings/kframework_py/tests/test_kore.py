from kframework_py.kore.syntax import Id

def test_id():
    i = Id("foo")

    a = i.value
    b = i.value

    assert a == "foo"

    # TODO: Fix this!
    # assert a is b

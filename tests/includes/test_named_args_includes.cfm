suiteBegin("Named Args from Includes");

include "helper_named_args.cfm";

// Direct call with named args from main script
assert("named args direct call", namedArgFunc(name="foo", value="bar"), "foo=bar");

// Call via function defined in include that uses named args
assert("named args from include function", callWithNamedArgs(), "hello=world");

// Named args where arg name matches var name
var name = "test";
var value = "data";
assert("named args same name as var", namedArgFunc(name=name, value=value), "test=data");

suiteEnd();

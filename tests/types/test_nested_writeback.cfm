<cfscript>
suiteBegin("Nested struct writeback through dynamic keys");

// Repro of the bug found in Taffy's cacheBeanMetaData: writing through
// `outer[memberAccessKey].child[memberAccessKey] = v` would lose the
// mutation because compile_expression_static didn't handle MemberAccess
// indexes during the writeback chain.
function nestedWritebackTest() {
    var local = {};
    local.endpoints = {};
    local.metaInfo = { uriRegex = "m" };
    local.f = { name = "get" };
    local.endpoints[local.metaInfo.uriRegex] = { methods = {} };
    local.endpoints[local.metaInfo.uriRegex].methods[local.f.name] = local.f.name;
    return local.endpoints;
}
ep = nestedWritebackTest();
assert("dynamic-outer/dynamic-inner write persists", structKeyList(ep["m"].methods), "get");
assertTrue("methods has 'get' key", structKeyExists(ep["m"].methods, "get"));
assert("methods.get value", ep["m"].methods["get"], "get");

// Also check ArrayAccess as outer index
function arrAccessOuter() {
    var local = {};
    local.cache = { items = [{ key = "k1" }] };
    local.store = {};
    local.store[local.cache.items[1].key] = { tags = {} };
    local.store[local.cache.items[1].key].tags["a"] = 1;
    return local.store;
}
s = arrAccessOuter();
assertTrue("nested array-access outer key persists", structKeyExists(s["k1"].tags, "a"));
assert("nested array-access value", s["k1"].tags["a"], 1);

suiteEnd();
</cfscript>

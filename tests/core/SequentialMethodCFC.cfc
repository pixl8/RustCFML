component {
    variables.encode = { string: function(d) { return chr(2) & d; } };

    function init() {
        // Sequential calls during pseudo-constructor — these used to
        // return empty after the first because the receiver `encode`
        // was being polluted with __variables via stale writeback.
        variables.a = encode.string("aaa");
        variables.b = encode.string("bbb");
        variables.c = encode.string("ccc");
        return this;
    }

    function results() {
        return { a: variables.a, b: variables.b, c: variables.c };
    }
}

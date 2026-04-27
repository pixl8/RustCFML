component {
    // Parent declares a helper struct in the variables scope; child body
    // references it during its own pseudo-constructor — Bug G regression.
    variables.encode = { string: function(d) { return "ENC:" & d; } };
}

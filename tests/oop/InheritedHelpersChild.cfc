component extends="InheritedHelpersParent" {
    // Touch parent's encode helper at page level — must resolve to the
    // parent's struct (Bug G: previously threw because parent's `variables`
    // wasn't visible until after child body completed).
    variables.dummyData = structNew();
    variables.dummyData.whatever = true;
    variables.dummyData.encoded = encode.string("payload");

    function get() {
        return variables.dummyData;
    }
}

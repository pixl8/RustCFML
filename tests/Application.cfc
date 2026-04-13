component {

    this.name = "RustCFMLTests";

    // Map "oop" to the tests/oop/ directory so createObject("component", "oop.Greeter") resolves
    this.mappings["/oop"] = getDirectoryFromPath(getCurrentTemplatePath()) & "oop/";

    // Map "tags" for any tag-based test includes
    this.mappings["/tags"] = getDirectoryFromPath(getCurrentTemplatePath()) & "tags/";

}

component {

    function callLine(string text="default") {
        return this.svc.line(arguments.text);
    }

    function callGreenLine(string text="default") {
        return this.svc.greenLine(arguments.text);
    }

    function getMeta() {
        return getComponentMetadata(this);
    }

}

component {
    function getMyProp() {
        return myProp;
    }

    function getThisProp() {
        return this.myProp;
    }

    function getThisKeys() {
        return structKeyList(this);
    }
}

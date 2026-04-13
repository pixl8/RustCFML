component accessors="true" {

    property name="packageService" inject="PackageService";
    property name="print" inject="print";
    property name="configService" inject="ConfigService" hint="Configuration manager";
    property name="greeting" type="string" default="Hello";

    function init() {
        return this;
    }

    function getServiceName() {
        return "InjectableService";
    }
}

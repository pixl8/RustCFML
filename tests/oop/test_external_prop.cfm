suiteBegin("External Property Access on CFC");

obj = createObject("component", "oop.ExternalPropCFC");

// Set a property from outside
obj.myProp = "hello";

// Property should be accessible from outside
assert("myProp from outside", obj.myProp, "hello");

// this.myProp should be accessible inside a method (Lucee-verified)
assert("this.myProp in method", obj.getThisProp(), "hello");

// Unscoped myProp should NOT resolve — Lucee throws "variable doesn't exist"
assertThrows("unscoped myProp throws", function() { obj.getMyProp(); });

// Multiple external properties
obj.anotherProp = 42;
assert("multiple external props", obj.anotherProp, 42);

suiteEnd();

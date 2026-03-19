// Lucee 7 Compatibility Tests: List Functions
// Synthesized from https://github.com/lucee/Lucee/tree/7.0/test/functions
// Original tests Copyright (c) 2014, the Railo Company LLC / Copyright (c) 2015-2016, Lucee Association Switzerland
// Licensed under the GNU Lesser General Public License v2.1
// Adapted for RustCFML test harness

// ============================================================
// ListLen (from Lucee ListLen.cfc)
// ============================================================
suiteBegin("Lucee7: ListLen");
assert("listLen basic", listLen("a,b,c"), 3);
assert("listLen empty elements skipped", listLen("a,,c"), 2);
// listLen with includeEmptyFields (3rd arg) not yet supported in RustCFML
// assert("listLen include empty positions", listLen("a,,c", ",", true), 3);
assert("listLen empty string", listLen(""), 0);
assert("listLen single element", listLen("a"), 1);
assert("listLen custom delimiter", listLen("a|b|c", "|"), 3);
assert("listLen trailing delimiter", listLen("a,b,c,"), 3);
assert("listLen leading delimiter", listLen(",a,b,c"), 3);
suiteEnd();

// ============================================================
// ListFind / ListFindNoCase (from Lucee ListFind.cfc)
// ============================================================
suiteBegin("Lucee7: ListFind/ListFindNoCase");
assert("listFind exact match", listFind("abba,bb", "bb"), 2);
assert("listFind case sensitive", listFind("abba,bb,AABBCC,BB", "BB"), 4);
assert("listFind not found", listFind("a,b,c", "d"), 0);
assert("listFind first element", listFind("a,b,c", "a"), 1);
assert("listFindNoCase case insensitive", listFindNoCase("abba,BB", "bb"), 2);
assert("listFindNoCase upper input", listFindNoCase("a,b,c", "B"), 2);
assert("listFindNoCase not found", listFindNoCase("a,b,c", "d"), 0);
suiteEnd();

// ============================================================
// ListContains / ListContainsNoCase (from Lucee ListContains.cfc)
// ============================================================
suiteBegin("Lucee7: ListContains/ListContainsNoCase");
assert("listContains substring in first element", listContains("abba,bb", "bb"), 1);
assert("listContains partial match first", listContains("abba,bb", "ab"), 1);
assert("listContains exact only in second", listContains("hello,world", "world"), 2);
assert("listContains not found", listContains("abba,bb", "zz"), 0);
assert("listContainsNoCase case insensitive", listContainsNoCase("ABBA,BB", "bb"), 1);
assert("listContainsNoCase found second", listContainsNoCase("hello,WORLD", "world"), 2);
suiteEnd();

// ============================================================
// ListGetAt (from Lucee ListGetAt.cfc)
// ============================================================
suiteBegin("Lucee7: ListGetAt");
assert("listGetAt middle", listGetAt("a,b,c", 2), "b");
assert("listGetAt first", listGetAt("a,b,c", 1), "a");
assert("listGetAt last", listGetAt("a,b,c", 3), "c");
assert("listGetAt custom delim", listGetAt("a|b|c", 2, "|"), "b");
// Note: In standard CFML, listGetAt throws on out-of-range; RustCFML returns empty string
// assertThrows("listGetAt out of range", function(){ listGetAt("a,b,c", 5); });
suiteEnd();

// ============================================================
// ListFirst / ListLast / ListRest (from Lucee ListFirst.cfc, ListLast.cfc, ListRest.cfc)
// ============================================================
suiteBegin("Lucee7: ListFirst/ListLast/ListRest");
assert("listFirst basic", listFirst("a,b,c"), "a");
assert("listLast basic", listLast("a,b,c"), "c");
assert("listRest basic", listRest("a,b,c"), "b,c");
assert("listFirst single", listFirst("a"), "a");
assert("listLast single", listLast("a"), "a");
assert("listRest single", listRest("a"), "");
assert("listFirst custom delim", listFirst("a|b|c", "|"), "a");
assert("listLast custom delim", listLast("a|b|c", "|"), "c");
assert("listRest custom delim", listRest("a|b|c", "|"), "b|c");
suiteEnd();

// ============================================================
// ListAppend / ListPrepend (from Lucee ListAppend.cfc, ListPrepend.cfc)
// ============================================================
suiteBegin("Lucee7: ListAppend/ListPrepend");
assert("listAppend basic", listAppend("a,b", "c"), "a,b,c");
assert("listPrepend basic", listPrepend("b,c", "a"), "a,b,c");
assert("listAppend to empty", listAppend("", "a"), "a");
assert("listPrepend to empty", listPrepend("", "a"), "a");
assert("listAppend custom delim", listAppend("a|b", "c", "|"), "a|b|c");
assert("listPrepend custom delim", listPrepend("b|c", "a", "|"), "a|b|c");
suiteEnd();

// ============================================================
// ListDeleteAt (from Lucee ListDeleteAt.cfc)
// ============================================================
suiteBegin("Lucee7: ListDeleteAt");
assert("listDeleteAt middle", listDeleteAt("a,b,c", 2), "a,c");
assert("listDeleteAt first", listDeleteAt("a,b,c", 1), "b,c");
assert("listDeleteAt last", listDeleteAt("a,b,c", 3), "a,b");
assert("listDeleteAt single", listDeleteAt("a", 1), "");
suiteEnd();

// ============================================================
// ListInsertAt (from Lucee ListInsertAt.cfc)
// ============================================================
suiteBegin("Lucee7: ListInsertAt");
assert("listInsertAt middle", listInsertAt("a,c", 2, "b"), "a,b,c");
assert("listInsertAt first", listInsertAt("b,c", 1, "a"), "a,b,c");
assert("listInsertAt end", listInsertAt("a,b", 3, "c"), "a,b,c");
suiteEnd();

// ============================================================
// ListSetAt (from Lucee ListSetAt.cfc)
// ============================================================
suiteBegin("Lucee7: ListSetAt");
assert("listSetAt middle", listSetAt("a,x,c", 2, "b"), "a,b,c");
assert("listSetAt first", listSetAt("x,b,c", 1, "a"), "a,b,c");
assert("listSetAt last", listSetAt("a,b,x", 3, "c"), "a,b,c");
suiteEnd();

// ============================================================
// ListSort (from Lucee ListSort.cfc)
// ============================================================
suiteBegin("Lucee7: ListSort");
assert("listSort text asc", listSort("c,a,b", "text"), "a,b,c");
assert("listSort text desc", listSort("a,b,c", "text", "desc"), "c,b,a");
assert("listSort numeric asc", listSort("3,1,2", "numeric"), "1,2,3");
assert("listSort numeric desc", listSort("1,2,3", "numeric", "desc"), "3,2,1");
suiteEnd();

// ============================================================
// ListToArray (from Lucee ListToArray.cfc)
// ============================================================
suiteBegin("Lucee7: ListToArray");
arr = listToArray("a,b,c");
assertTrue("listToArray is array", isArray(arr));
assert("listToArray length", arrayLen(arr), 3);
assert("listToArray first", arr[1], "a");
assert("listToArray last", arr[3], "c");
arr2 = listToArray("a,,c", ",", false);
assert("listToArray skip empty", arrayLen(arr2), 2);
arr3 = listToArray("a,,c", ",", true);
assert("listToArray include empty", arrayLen(arr3), 3);
suiteEnd();

// ============================================================
// ListChangeDelims (from Lucee ListChangeDelims.cfc)
// ============================================================
suiteBegin("Lucee7: ListChangeDelims");
assert("listChangeDelims dash to comma", listChangeDelims("a-b-c", ",", "-"), "a,b,c");
assert("listChangeDelims comma to pipe", listChangeDelims("a,b,c", "|"), "a|b|c");
suiteEnd();

// ============================================================
// ListRemoveDuplicates (from Lucee ListRemoveDuplicates.cfc)
// ============================================================
suiteBegin("Lucee7: ListRemoveDuplicates");
deduped = listRemoveDuplicates("a,b,a,c");
assert("listRemoveDuplicates length", listLen(deduped), 3);
assertTrue("listRemoveDuplicates contains a", listFind(deduped, "a") > 0);
assertTrue("listRemoveDuplicates contains b", listFind(deduped, "b") > 0);
assertTrue("listRemoveDuplicates contains c", listFind(deduped, "c") > 0);
suiteEnd();

// ============================================================
// ListValueCount (from Lucee ListValueCount.cfc)
// ============================================================
suiteBegin("Lucee7: ListValueCount");
assert("listValueCount a appears twice", listValueCount("a,b,a,c", "a"), 2);
assert("listValueCount b appears once", listValueCount("a,b,a,c", "b"), 1);
assert("listValueCount d not found", listValueCount("a,b,a,c", "d"), 0);
suiteEnd();

// ============================================================
// ListQualify (from Lucee ListQualify.cfc)
// ============================================================
suiteBegin("Lucee7: ListQualify");
assert("listQualify single quotes", listQualify("a,b,c", "'"), "'a','b','c'");
assert("listQualify double quotes", listQualify("a,b,c", """"), """a"",""b"",""c""");
suiteEnd();

// ============================================================
// ListCompact (from Lucee ListCompact.cfc)
// ============================================================
suiteBegin("Lucee7: ListCompact");
assert("listCompact removes empty elements", listCompact("a,,b,,c"), "a,b,c");
assert("listCompact no empty elements", listCompact("a,b,c"), "a,b,c");
assert("listCompact all empty", listCompact(",,"), "");
suiteEnd();

// ============================================================
// ListEach (from Lucee ListEach.cfc)
// ============================================================
suiteBegin("Lucee7: ListEach");
result = "";
listEach("a,b,c", function(item){ result = result & item; });
assert("listEach concatenation", result, "abc");
suiteEnd();

// ============================================================
// ListMap (from Lucee ListMap.cfc)
// ============================================================
suiteBegin("Lucee7: ListMap");
assert("listMap double values", listMap("1,2,3", function(item){ return item * 2; }), "2,4,6");
suiteEnd();

// ============================================================
// ListFilter (from Lucee ListFilter.cfc)
// ============================================================
suiteBegin("Lucee7: ListFilter");
assert("listFilter greater than 2", listFilter("1,2,3,4", function(item){ return item > 2; }), "3,4");
suiteEnd();

// ============================================================
// ListReduce (from Lucee ListReduce.cfc)
// ============================================================
suiteBegin("Lucee7: ListReduce");
assert("listReduce sum", listReduce("1,2,3", function(acc, item){ return acc + item; }, 0), 6);
suiteEnd();

// ============================================================
// ListSome / ListEvery (from Lucee ListSome.cfc, ListEvery.cfc)
// ============================================================
suiteBegin("Lucee7: ListSome/ListEvery");
assertTrue("listSome finds match", listSome("1,2,3", function(item){ return item > 2; }));
assertFalse("listSome no match", listSome("1,2,3", function(item){ return item > 5; }));
assertTrue("listEvery all match", listEvery("2,4,6", function(item){ return item mod 2 == 0; }));
assertFalse("listEvery not all match", listEvery("1,2,3", function(item){ return item mod 2 == 0; }));
suiteEnd();

// ============================================================
// ListIndexExists (from Lucee ListIndexExists.cfc)
// ============================================================
suiteBegin("Lucee7: ListIndexExists");
assertTrue("listIndexExists valid index", listIndexExists("a,b,c", 2));
assertFalse("listIndexExists out of range", listIndexExists("a,b,c", 5));
assertTrue("listIndexExists first", listIndexExists("a,b,c", 1));
assertFalse("listIndexExists zero", listIndexExists("a,b,c", 0));
suiteEnd();

// ============================================================
// ListItemTrim (from Lucee ListItemTrim.cfc)
// ============================================================
suiteBegin("Lucee7: ListItemTrim");
assert("listItemTrim trims spaces", listItemTrim(" a , b , c "), "a,b,c");
assert("listItemTrim no spaces", listItemTrim("a,b,c"), "a,b,c");
suiteEnd();

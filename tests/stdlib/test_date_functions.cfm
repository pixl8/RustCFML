<cfscript>
suiteBegin("Date Functions");

// --- now ---
assertTrue("now() returns a date", isDate(now()));

// --- createDate ---
d = createDate(2024, 6, 15);
assert("year of createDate", year(d), 2024);
assert("month of createDate", month(d), 6);
assert("day of createDate", day(d), 15);

// --- createDateTime ---
dt = createDateTime(2024, 6, 15, 10, 30, 0);
assert("hour of createDateTime", hour(dt), 10);
assert("minute of createDateTime", minute(dt), 30);
assert("second of createDateTime", second(dt), 0);

// --- dateAdd ---
added = dateAdd("d", 1, createDate(2024, 1, 31));
assert("dateAdd day wraps to Feb", day(added), 1);
assert("dateAdd month wraps to Feb", month(added), 2);

// --- dateDiff ---
assert("dateDiff days", dateDiff("d", createDate(2024, 1, 1), createDate(2024, 1, 31)), 30);

// --- dateFormat ---
assert("dateFormat mm/dd/yyyy", dateFormat(createDate(2024, 6, 15), "mm/dd/yyyy"), "06/15/2024");

// --- dayOfWeek ---
dow = dayOfWeek(createDate(2024, 1, 1));
assertTrue("dayOfWeek returns 1-7", dow >= 1 && dow <= 7);

// --- daysInMonth ---
assert("daysInMonth Feb 2024 (leap)", daysInMonth(createDate(2024, 2, 1)), 29);

// --- isDate ---
assertTrue("isDate(now()) is true", isDate(now()));
assertFalse("isDate('not a date') is false", isDate("not a date"));

// --- year(now()) ---
assertTrue("year(now()) > 2023", year(now()) > 2023);

// --- getTickCount ---
assertTrue("getTickCount() > 0", getTickCount() > 0);

suiteEnd();
</cfscript>

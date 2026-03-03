<cfscript>
suiteBegin("BitMask Functions");

// bitMaskRead(number, start, length)
assert("bitMaskRead basic", bitMaskRead(255, 0, 4), 15);
assert("bitMaskRead offset", bitMaskRead(255, 4, 4), 15);
assert("bitMaskRead partial", bitMaskRead(170, 0, 4), 10);
assert("bitMaskRead single bit", bitMaskRead(8, 3, 1), 1);
assert("bitMaskRead zero", bitMaskRead(0, 0, 8), 0);

// bitMaskSet(number, mask, start, length)
assert("bitMaskSet basic", bitMaskSet(0, 15, 0, 4), 15);
assert("bitMaskSet offset", bitMaskSet(0, 15, 4, 4), 240);
assert("bitMaskSet replace", bitMaskSet(255, 0, 0, 4), 240);
assert("bitMaskSet single bit", bitMaskSet(0, 1, 3, 1), 8);

// bitMaskClear(number, start, length)
assert("bitMaskClear low bits", bitMaskClear(255, 0, 4), 240);
assert("bitMaskClear high bits", bitMaskClear(255, 4, 4), 15);
assert("bitMaskClear single bit", bitMaskClear(8, 3, 1), 0);
assert("bitMaskClear no effect", bitMaskClear(240, 0, 4), 240);

suiteEnd();
</cfscript>

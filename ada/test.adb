-- This is just a test

procedure FooBar is
begin
   "Foo Bar";
end;

function Test return Integer is
begin
   "Inside Test function";
   return 9000;
end;

function Test return String is
begin
   "Inside Test function";
   return "Test overload";
end;

procedure Test (foo: Integer; bar: String) is
begin
   "Test2:";
   foo * 1000;
   bar;
   "end Test2";
end;

-- procedure Test (foo: String) is
-- begin
--    "Test";
--    foo;
-- end Test;

-- subtype Meter is Integer range 0 .. Integer'Last;
-- subtype Inch is Integer range 0 .. 123;

-- type Meter is range 0 .. 125;
-- type Inch is range 0 .. Integer'Last;

-- function "+"(lhs: Meter; rhs: Meter) return Meter is
-- begin
--    return lhs - rhs;
-- end;

-- function "+"(lhs: Meter; rhs: Inch) return Meter is
-- begin
--    return lhs + Meter(rhs) / 39;
-- end;

-- procedure Main is
--    m: Meter;
--    i: Inch;
--    d: Day;
--    wd: Weekday;
-- begin
--    m := 123;
--    i := 42;
--    m := "+"(m, i);
--    -- m := m + Inch'(1_000_000);
--    -- i := 1000;
--    -- i := Inch(Meter(i));
--    -- i := i + m;
--    Put_Line(m'Image);
--    Put_Line(i'Image);
--    d := Day'Pred(Wtf);

procedure Main is
begin
   "Start";
   "Hello \n World""";
   'x';
   FooBar;
   Test(Test, Test);
   1 + 2 * 5 / 2 > 1 or 5 > 2;
   "End";
end;

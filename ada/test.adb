-- This is just a test

procedure FooBar is
begin
   Print("Foo Bar");
end;

procedure Test is
begin
   Print("Inside Test procedure");
end;

function Test return Integer is
begin
   Print("Inside Test int function");
   return 9000;
end;

function Test return String is
begin
   Print("Inside Test str function");
   return "Test overload";
end;

procedure Test (foo: Integer; bar: String) is
begin
   Print("Test2:");
   Print(foo * 1000);
   Print(bar);
   Print("end Test2");
end;


function ReturnCheck return Integer is
begin
   return Test;
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

type Day is (Mon, Tue, Wed, Thu, Fri, Sat, Sun);
type Weekday is (Mon, Tue, Wed, Thu, Fri);
-- subtype Weekday is Day range Mon .. Fri;

procedure Main is
   i: Integer;
   d: Day;
   wd: Weekday;
begin
   Print("Start");
   Print("Hello \n World""");
   Print('x');
   i := 123;
   Print(i);
   i := Test;
   Print(i);
   d := Sat;
   Print(d);
   wd := Thu;
   Print(wd);
   Test;
   FooBar;
   Test(Test, Test);
   Print(1 + 2 * 5 / 2 > 1 or 5 > 2);
   Print(ReturnCheck);
   Print("End");
end;

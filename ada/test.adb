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
   Print(foo);
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

type Meter is range 0 .. 1_000_000;
type Inch is range 0 .. 123;
type Byte is mod 256;
type Minute is mod 60;

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
type Weekend is (Sat, Sun);
subtype Weekday is Day range Mon .. Fri;

procedure Main is
   d: Day;
   wd: Weekday;
   m: Meter;
   i: Inch;
   min: Minute;
   b: Boolean;
begin
   Print("Start");
   Print("Hello \n World""");
   Print('x');
   d := Sat;
   Print(d);
   wd := Thu;
   Print(wd);
   d := wd;
   Print(d);
   wd := d;
   Print(wd);
   Test;
   FooBar;
   Test(Test, Test);
   -- Print(1 + 2 * 5 / 2 > 1 or 5 > 2);
   Print(ReturnCheck);

   m := 5;
   m := m + 3;
   Print(m);

   i := 3;
   i := i + 120;
   Print(i);

   Print(Inch'(123));
   Print(Meter'(123));
   Print(Integer'(123));
   Print(Day'(Sat));
   Print(Weekend'(Sat));
   -- Print(Weekday'(Sat)); --> Constraint_Error


   min := 5;
   Print(min + 5 / 3 * 59);

   b := false;
   Print(b);

   b := true;
   Print(b);

   b := i < 5;
   Print(b);

   b := b < false;
   Print(b);

   Print("End");
end;

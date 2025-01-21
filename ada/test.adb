-- This is just a test

type Month_Name is (Jan, Feb, Mar, Apr, May, Jun, Jul, Aug, Sep, Oct, Nov, Dec);

subtype Day is Integer range 1 .. 31;
subtype Year is Integer range 0 .. 4000;

type Date is
   record
      Day : Day;
      Month : Month_Name;
      Year : Year;
   end record;

procedure Next(d: in out Date) is
begin
   d.Day := d.Day + 1;
   Print(d);
end;

function Next(d: Date) return Date is
   r: Date;
begin
   r := d;
   Next(r);
   return r;
end;

procedure Main is
   d : Date;
begin
   Print("Start");

   d := (21, Jan, 2025);
   Print(d);

   d := (Day => 21, Month => Jan, Year => 2025);
   Print(d);

   d := (Day => 21, Month => Jan, others => 2025);
   Print(d);

   for i in 1 .. 10 loop
      d := Next(d);
      Print(d);
      Print(i);
   end loop;

   Print("End");
end;

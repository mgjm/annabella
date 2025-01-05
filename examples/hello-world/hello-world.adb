with Ada.Text_IO;
with Test;
with Interfaces;

procedure HelloWorld is
   unused : Interfaces.Integer_32;
begin
   -- TODO: variables do not work yet
   -- unused := Test.Test;
   -- unused := Test.Test(123);
   Test.Test2;
   Ada.Text_IO.Put_Line("Hello World String");
   Ada.Text_IO.Put_Line(42);
   Ada.Text_IO.Put_Line(Test.GetMessage);
end HelloWorld;

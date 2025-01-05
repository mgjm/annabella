with Ada.Text_IO;
with Test;
with Interfaces;

procedure HelloWorld is
   unused : Interfaces.Integer_32;
begin
   unused := Test.Test;
   Ada.Text_IO.Put_Line(unused);
   -- TODO: function overloading not yet supported
   -- unused := Test.Test(42);
   Test.Test2;
   Ada.Text_IO.Put_Line("Hello World String");
   Ada.Text_IO.Put_Line(42);
   Ada.Text_IO.Put_Line(Test.GetMessage);
end HelloWorld;

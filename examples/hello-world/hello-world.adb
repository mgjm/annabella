with Ada.Text_IO;
with Test;
with Interfaces;

procedure HelloWorld is
   unused : Interfaces.Integer_32;
   str_var : String;
begin
   unused := Test.Test;
   Ada.Text_IO.Put_Line(unused);
   -- TODO: function overloading not yet supported
   -- unused := Test.Test(42);
   Test.Test2;
   Ada.Text_IO.Put_Line("Hello World String");
   Ada.Text_IO.Put_Line(42);
   str_var := "Hello World Variable";
   Ada.Text_IO.Put_Line(str_var);
   Ada.Text_IO.Put_Line(Test.GetMessage);
   Test.PrintTemperature(0);
   Test.PrintTemperature(2);
   Test.PrintTemperature(12);
   Test.PrintTemperature(22);
   Test.PrintTemperature(32);
   Test.PrintTemperature(42);
end HelloWorld;

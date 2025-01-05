with Ada.Text_IO;
with Interfaces;

package body Test is
   function Test return Interfaces.Integer_32 is
   begin
      Ada.Text_IO.Put_Line("Test.Test called");
      return 123;
   end Test;

   procedure Test2 is
   begin
      Ada.Text_IO.Put_Line("Test.Test2 called");
   end Test2;

   function GetMessage return string is
   begin
      Ada.Text_IO.Put_Line("Test.GetMessage called");
      return "Hello World from Ada!";
   end GetMessage;

   procedure PrintTemperature(Temperature : Interfaces.Integer_32) is
   begin
      Ada.Text_IO.Put_Line(Temperature);
   end PrintTemperature;
end Test;

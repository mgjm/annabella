with Ada.Text_IO;
with Interfaces;

package body Test is
   function Test return Interfaces.Integer_32 is
   begin
      Ada.Text_IO.Put_Line("Test.Test called");
      Test2;
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

      if Temperature > 40 then
          Ada.Text_IO.Put_Line("Wow!");
          Ada.Text_IO.Put_Line("It's extremely hot");
      elsif Temperature > 30 then
          Ada.Text_IO.Put_Line("It's hot");
      elsif Temperature > 20 then
          Ada.Text_IO.Put_Line("It's warm");
      elsif Temperature > 10 then
          Ada.Text_IO.Put_Line("It's cool");
      elsif Temperature > 0 then
          Ada.Text_IO.Put_Line("It's cold");
      else
          Ada.Text_IO.Put_Line("It's freezing");
      end if; 
   end PrintTemperature;
end Test;

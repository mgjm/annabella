with Interfaces;

package Test is
   function Test return Interfaces.Integer_32;
   procedure Test2;
   function GetMessage return string;
   procedure PrintTemperature(Temperature : Interfaces.Integer_32);
end Test;

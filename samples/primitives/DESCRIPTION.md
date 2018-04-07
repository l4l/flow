Constant
==

Features Used
===
* Context Flow
* Library Functions used (`add` and `stdout` from `flowstdlib`)
* Value used (with an initial value set)
* Constant Value used
* Connections between functions
* Two functions of the same name in the same flow, distinguished by `alias`
* `switch` function to stop or pass a data flow based on another one
* `compare` function to produce outputs based on comparing two input values

Description
===
A flow that takes a value and a constant and adds them, and then takes the result and adds 
it to the constant again and then printed the final value to stdout. It also uses the switch function
to stop a flow with a false value, and compares the result of the add to a value and print out to stdout
if it is greater or equal to it.
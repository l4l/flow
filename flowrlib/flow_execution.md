# Flow Execution

## Lazy Execution
Execution should be as lazy as possible.

The only thing that determines if a function is run is the availability at it's inputs of the data
it needs to run, and the ability to produce the result at it's output by having the output free.

If the output is free because a second function is running and hasn't consumed it's input, then 
the first function will be blocked and computing resources will be used on the functions that most 
need it. When they complete and produce an output, that output may satisfy another functions's 
inputs which in turn will run, and so on and so forth.

## Execution States
Runnables (Values and Functions) can be in one of two states:
- blocked (either pending an IO or output is full)
- runnable (inputs are satisfied, output is free and it can be run anytime)

## Value Rules
A Value has only one input, but that can be connected to and a value offered by multiple "writers".
It has only one output, but that can be connected to and listened on by multiple "listeners"

It stores a value, that can be initialized when the program is loaded to an initial value.

When the value is empty it can be updated.
When it is updated, the value is made available to all "listeners" at it's output.
While it stores a value, it cannot be updated, and any writer will be blocked until the value is 
consumed and can be updated again.

Each of the listeners can read or "consume" the value once.

When it has been consumed by all listeners, the value becomes empty (None) and can be updated again.
It does not become empty until all listeners have consumed the value.

You can think of Values as a FIFO of size 1 for each listener connected to it.

## Function Rules
The operation of a Function is similar to that of a value, except that it can have multiple inputs
and it is not run until they are all satisfied.

A function does not store any value or state, beyond making it's output available to listeners
asynchronously.

A Function can have zero or more inputs. 
Each input can be connected to and written to by multiple "writers".
It has only one output, but that can be connected to and listened to by multiple "listeners".

A Function can only run when a value is available at each of it's inputs and it's output is 
free to write to.. It is blocked from running until these conditions are met.

When a function runs, it produces an output that is made available to all "listeners" at it's output.

Each of the listeners can read or "consume" the output value once.

## Generalized Rules 
If we consider a value to be like a null function that does no calculation, but just passes the input 
value to it's outputs - then we can state some general rules that apply to the "running" of both.

- Can have zero or more inputs (max 1 for a Value)
- Each input can be connected to and values offered to it by multiple "writers".
- Has one output, that can be listened on by multiple "listeners".
- Can only be run (updated) when a value is available at each of the inputs and the output is 
free to write to. Is blocked from running until these conditions are met.
- When ran, it produces an output that is made available to all "listeners" at it's output.
- Each of the listeners can read or "consume" the output value only once.
- Once the output has been consumed (once) by all the listeners, then the output is free to be
written to again.

# Executuion Process
## Loading
All functions and values are loaded, and initially placed in "blocked" state.

## Initialization
Values are updated (ran) with initial values satisfied and hence they make their value available
at their output.

## Execution Loop
Outputs produced are made available to all connected inputs
Status of Functions/Values are updated based on availability of data on all inputs
Functions/Values with status "runnable" are run, producing outputs.


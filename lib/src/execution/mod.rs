pub mod entity;
pub mod function;
pub mod value;


/*

validate model (see check)

construct overall list of functions

construct list of connections

construct initial list of all functions able to produce output
    - start from external sources at level 0

do
    - identify all functions which receive input from active sources
    - execute all those functions
    - functions producing output added to list of active sources
while functions pending input

 */
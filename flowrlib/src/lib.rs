//! Runtime library for flow execution. This will be linked with code generated from a flow definition
//! to enable it to be compiled and ran as a native program.
#[macro_use]
extern crate log;
#[macro_use] extern crate serde_json;

extern crate simplog;
extern crate clap;

pub mod info;
pub mod execution;
pub mod startup;
pub mod runlist;
pub mod value;
pub mod implementation;
pub mod function;
pub mod runnable;
pub mod zero_fifo;
mod input;

pub mod env;
pub mod stdio;
pub mod file;
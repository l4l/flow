use std::panic;
use std::sync::mpsc::{Sender, Receiver};
use runlist::{RunSet, OutputSet};

/*
    Loop on calling thread:
        - receiving runsets to run from the receiver channel
        - execute the runset
        - send OutputSets in return
*/
pub fn executor(receiver: Receiver<RunSet>, sender: Sender<OutputSet>) {
    set_panic_hook();

    //  TODO have a special message to exit - or the channels gets closed on us?

    debug!("Starting execution loop");
    debug!("-----------------------------------------------------------------");
    while let Ok(runset) = receiver.recv() {
        // Get a runset (runnable id, implementation, inputs to use) and run it, it will
        // use the sender channel to send outputs back to the RunListServer
        runset.implementation.run(runset.id, runset.inputs, &sender);
    }

    debug!("Ended execution loop");
}

/*
    Replace the standard panic hook with one that just outputs the file and line of any runnable's
    runtime panic.
*/
fn set_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        if let Some(location) = panic_info.location() {
            error!("panic occurred in file '{}' at line {}", location.file(), location.line());
        } else {
            error!("panic occurred but can't get location information...");
        }
    }));
    debug!("Panic hook set to catch panics in runnables");
}
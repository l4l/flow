use std::panic;
use std::sync::mpsc::{Sender, Receiver};
use runlist::{RunSet, OutputSet};

/// The generated code for a flow consists of values and functions formed into a list of Runnables
/// that is served by a "runlist server" thread/process.
///
/// Flows should submit their list of runnables to the runlist server, get a Receiver from it
/// (on which to receive sets of inputs and implementations to run) and then start executing them
/// in a thread.
///
/// This execute() method is a convencience method that requests those sets of inputs and
/// implementations to run, and executes them.
///
/// # Example
/// ```
/// use flowrlib::runlist::RunList;
/// use flowrlib::execution::execute;
/// use std::process::exit;
///
/// let (receiver, sender) = RunList::connect(get_runnables());
/// execute(receiver, sender);
///
/// exit(0);
/// ```
pub fn execute(receiver: Receiver<RunSet>, sender: Sender<OutputSet>) {
    set_panic_hook();

    debug!("Starting execution loop");
    debug!("-----------------------------------------------------------------");
    while let Ok(runset) = receiver.recv() {
        // if after all is said and done it can run again, then add to the end of the ready list
        let run_again = runset.implementation.run(runset.id, runset.inputs, sender);

        // TODO move this to the end of the thread running the implementation?
        // TODO or send a message to runlist server to indicate this one has ran to completion
        // run_list.done(id, run_again);
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
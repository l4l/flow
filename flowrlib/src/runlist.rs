use runnable::Runnable;
use runnable::RunnableState;
use std::fmt;
use std::time::Instant;
use serde_json::Value as JsonValue;
use std::sync::mpsc::{Sender, Receiver};
use implementation::Implementation;
use implementation::RunAgainOption;
use implementation::RunAgainOption::{RunAgain, DontRunAgain};
use runnable::RunnableID;

pub struct Metrics {
    num_runnables: usize,
    dispatches: u32,
    outputs_sent: u32,
    start_time: Instant,
}

enum RunnableEvent {
    InitReady,
    // Has been initialized and is now ready to run
    InitNeedsInput,
    // Has been initiatized but needs input in order to be able to run
    DoneDead,
    // Done running, but doesn't want to run again
    DoneBlocked,
    // Done running, is blocked on output from running again
    DoneWaiting,
    // Done running, needs input in order to run again
    DoneReady,
    // Done running, has input and can output so can run again
    Dispatched,
    // Sent to be run
    Unblocked,
    // Was blocked but now output was freed
    InputNotBlocked,
    // Inputs became ready, not blocked on output
    InputBlocked,
    // Inputs became ready, blocked on output
    OutputSent,
    // Runnable sent an output but continues to run
}

impl Metrics {
    fn new(num_runnables: usize) -> Self {
        let now = Instant::now();
        Metrics {
            num_runnables,
            dispatches: 0,
            outputs_sent: 0,
            start_time: now,
        }
    }
}

impl fmt::Display for Metrics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let elapsed = self.start_time.elapsed();
        write!(f, "\t\tNumber of Runnables: \t{}\n", self.num_runnables)?;
        write!(f, "\t\tRunnable dispatches: \t{}\n", self.dispatches)?;
        write!(f, "\t\tOutputs sent: \t\t{}\n", self.outputs_sent)?;
        write!(f, "\t\tElapsed time(s): \t{:.*}\n", 9, elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9)
    }
}

/*
    RunList is a structure that maintains the state of all the runnables in the currently
    executing flow.

    A runnable maybe blocking multiple others trying to send data to it.
    Those others maybe blocked trying to send to multiple different runnables.

    runnables:
    A list of all the runnables that could be executed at some point.

    ready:
    A list of runnables who's inputs are ready to be dispatched.

    blocking:
    A list of tuples of runnable ids where first id is id of the runnable data is trying to be sent
    to, and the second id is the id of the runnable trying to send data.
*/
pub struct RunList<'a> {
    runnables: Vec<&'a mut Runnable>,
    ready: Vec<RunnableID>,
    blocking: Vec<(RunnableID, RunnableID)>,
    // blocking_id, blocked_id
    metrics: Metrics,
    sender: Sender<RunSet<'a>>,
    receiver: Receiver<OutputSet>,
}

// Set of information needed to run an implementation
pub struct RunSet<'a> {
    pub id: RunnableID,
    pub implementation: &'a Implementation,
    pub inputs: Vec<Vec<JsonValue>>,
}

// Set of information output by an Implementation
pub struct OutputSet {
    pub from: RunnableID,
    pub output: JsonValue,
    pub done: bool,
    pub run_again: RunAgainOption,
}

impl<'a> RunList<'a> {
    fn new(sender: Sender<RunSet<'a>>, receiver: Receiver<OutputSet>, runnables: Vec<&'a mut Runnable>) -> Self {
        let len = runnables.len();
        RunList {
            runnables,
            ready: Vec::<RunnableID>::new(),
            blocking: Vec::<(RunnableID, RunnableID)>::new(),
            metrics: Metrics::new(len),
            sender,
            receiver,
        }
    }

    /*
        This initializes each runnable in the list by calling it's `init` method.
        The `init` method returns a boolean to indicate that it's inputs are fulfilled (or not).
        This information is added to the RunList to control the readyness of  the Runnable to be executed.
    */
    fn init(&mut self) {
        debug!("Initializing runnables in the runlist");
        for runnable in &mut self.runnables {
            debug!("\tInitializing runnable #{} '{}'", runnable.id(), runnable.name());

            if runnable.init() { // Inputs are ready
                runnable.set_state(RunnableState::Ready);
            } else {
                runnable.set_state(RunnableState::Waiting);
            }
        }
    }

    /*
        Print out a simple debug of the various lists in the runlist
    */
    fn debug(&self) {
        debug!("Dispatch count: {}", self.metrics.dispatches);
        debug!("         Ready: {:?}", self.ready);
        debug!("      Blocking: {:?}", self.blocking);
        debug!("-------------------------------------");
    }

    /*
        Call this when a flow ends, to print out metrics related to its execution
    */
    fn flow_end(&self) {
        self.debug();
        debug!("Metrics: \n {}", self.metrics);
    }

    /*
        Get a mutable reference to a Runnable based on it's id (index in the runnables array)
    */
    fn get(&'a mut self, id: RunnableID) -> &mut Runnable {
        *(self.runnables.get_mut(id).unwrap())
    }

    /*
        Take the next ready runnable (id) off the ready list and return it
    */
    pub fn next_ready(&mut self) -> Option<RunnableID> {
        self.ready.pop()
    }

    /*
        Register the event produced by a Runnable's inputs being ready
    */
    fn inputs_ready(&'a mut self, id: RunnableID) -> RunnableEvent {
        debug!("\t\t\tRunnable #{} inputs are ready", id);
        if self.is_blocked(id) {
            RunnableEvent::InputBlocked
        } else {
            RunnableEvent::Unblocked
        }
    }

    /*
        Save the fact that the runnable 'blocked_id' is blocked on it's output by 'blocking_id'
    */
    // ADM TODO make this a state transition to blocked on output?
    fn blocked_by(&mut self, blocking_id: RunnableID, blocked_id: RunnableID) {
        // avoid deadlocks by a runnable blocking itself
        if blocked_id != blocking_id {
            debug!("\t\t\tRunnable #{} is now blocked on output by Runnable #{}", &blocked_id, &blocking_id);
            self.blocking.push((blocking_id, blocked_id));
        }
    }

    /*
        for all runnables that were blocked trying to send to blocker_id, send unblocked event
        remove entries in the blocking list where the blocking_id matches the blocker_id
    */
    fn unblock_senders_to(blocking: &mut Vec<(RunnableID, RunnableID)>, blocker_id: RunnableID) {
        if !blocking.is_empty() {
            let mut unblocked_list: Vec<RunnableID> = vec!();

            // TODO optimize this like is_blocked() needs
            for &mut (blocking_id, blocked_id) in blocking {
                if blocking_id == blocker_id {
                    debug!("\t\tRunnable #{} <-- #{} - removing block", blocking_id, blocked_id);
                    unblocked_list.push(blocked_id);
                }
            }

            // remove all entries from the blocking list with this blocker_id as they will be unblocked
            blocking.retain(|&(blocking_id, _blocked_id)| blocking_id != blocker_id);

            // for all the runnables previously blocked, and now are unblocked, send unblocked event
//            for unblocked in unblocked_list {
            // TODO solve problem of calling this in a loop
//                self.event(unblocked, RunnableEvent::Unblocked);
//            }
        }
    }

    /*
        An event has occurred related to runnable with RunnableID 'id'
        Take the actions necessary related to the state transition, calculate the new state
        and set the runnable to that new state.
    */
    fn event(ready: &mut Vec<RunnableID>, blocking: &mut Vec<(RunnableID, RunnableID)>,
             metrics: &mut Metrics, runnable: &mut Runnable, id: RunnableID, event: RunnableEvent) {
        {
            match event {
                RunnableEvent::DoneReady => ready.push(id),
                RunnableEvent::InitReady => ready.push(id),
                RunnableEvent::DoneDead => debug!("\tRunnable #{} completed", id),
                RunnableEvent::Dispatched => {
                    Self::unblock_senders_to(blocking, id);
                    metrics.dispatches += 1;
                }
                RunnableEvent::OutputSent => {
                    metrics.outputs_sent += 1;
                }
                _ => {}
            }
        }

        let new_state;
        {
            let _current_state = runnable.get_state();
            new_state = RunnableState::Init; // TODO calculate new state
        }
        runnable.set_state(new_state);
    }

    fn runset(runnable: &mut Runnable, id: RunnableID) -> RunSet {
        let inputs = runnable.get_inputs();
        let implementation = runnable.implementation();
        let name = runnable.name();
        debug!("Runnable #{} '{}' being dispatched with inputs: {:?}", id, name, inputs);
        RunSet { id, implementation, inputs }
    }

    /*
        A runnable has become able to be run, run the implementation on it's input data
            - gather it's inputs (marking them as consumed)
            - unblock others attempting to send to it
            - get the implementation to run on those inputs
            - send the id, implementation and inputs on the channel for dispatching
    */
    pub fn dispatch(&'a mut self, id: RunnableID) {
        let runnable = self.get(id);
        let runset = Self::runset(runnable, id);

        Self::event(&mut self.ready, &mut self.blocking, &mut self.metrics, runnable,
                    id, RunnableEvent::Dispatched);

        // Send the runset to an executor to be run
        self.sender.send(runset);
    }

    // TODO ADM change this to just check the current state - as an optimization
    /*
        See if a runnable is blocked on output
    */
    fn is_blocked(&self, id: RunnableID) -> bool {
        for &(_blocking_id, blocked_id) in &self.blocking {
            if blocked_id == id {
                return true;
            }
        }
        false
    }

    /*
        - receive the output set sent from a running runnable on another thread via the channel
        - send the output to all destination IOs on runnables it should be sent to
        - mark the source runnable as blocked because those others must consume the output // TODO
        - update state of other runnables the outputs have  been sent to
    */
    fn process_output_set(&'a mut self, output_set: OutputSet) {
        let runnable = self.get(output_set.from);

        for &(output_route, destination_id, io_number) in runnable.output_destinations() {
            let destination = self.get(destination_id);
            let output_value = output_set.output.pointer(output_route).unwrap();
            debug!("\t\tRunnable #{} '{}{}' sending output '{}' to Runnable #{} '{}' input #{}",
                   runnable.id(), runnable.name(), output_route, output_value, &destination_id,
                   destination.name(), &io_number);
            destination.write_input(io_number, output_value.clone());

            if destination.input_full(io_number) {
                self.blocked_by(destination_id, output_set.from);
            }

            if destination.inputs_ready() {
                Self::event(&mut self.ready, &mut self.blocking, &mut self.metrics, runnable,
                            destination_id, self.inputs_ready(destination_id));
            }
        }

        Self::event(&mut self.ready, &mut self.blocking, &mut self.metrics, runnable,
                    output_set.from, RunnableEvent::OutputSent);

        if output_set.done {
            let event = match output_set.run_again {
                RunAgain => {
                    if runnable.inputs_ready() {
                        if self.is_blocked(output_set.from) {
                            RunnableEvent::DoneBlocked
                        } else {
                            RunnableEvent::DoneReady
                        }
                    } else {
                        RunnableEvent::DoneWaiting
                    }
                },
                DontRunAgain => RunnableEvent::DoneDead,
            };
            Self::event(&mut self.ready, &mut self.blocking, &mut self.metrics, runnable,
                        output_set.from, event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RunList, RunSet};
    use super::Runnable;
    use runnable::RunnableState;
    use runnable::RunnableState::Init;
    use serde_json;
    use serde_json::Value as JsonValue;
    use super::super::implementation::{Implementation, RunAgainOption::RunAgain};
    use super::OutputSet;
    use super::RunnableID;
    use std::sync::mpsc::{Sender, Receiver};
    use std::sync::mpsc;

    struct TestImplementation;

    impl Implementation for TestImplementation {
        fn run(&self, id: RunnableID, inputs: Vec<Vec<JsonValue>>, tx: &Sender<OutputSet>) {
// get the input value
            let input = inputs.get(0).unwrap().get(0).unwrap().clone();

// Send the input directly as the output
            self.send_output(id, &tx, input, true /* done */, RunAgain);
        }
    }

    struct TestRunnable<'a> {
        id: RunnableID,
        number_of_inputs: usize,
        destinations: Vec<(&'static str, RunnableID, usize)>,
        implementation: &'a Implementation,
        blocked_on_output: bool,
        state: RunnableState,
    }

    impl<'a> TestRunnable<'a> {
        fn new(id: RunnableID, number_of_inputs: usize, destinations: Vec<(&'static str, RunnableID, usize)>) -> TestRunnable {
            TestRunnable {
                id,
                number_of_inputs,
                destinations,
                implementation: &TestImplementation,
                blocked_on_output: false,
                state: Init,
            }
        }
    }

    impl<'a> Runnable for TestRunnable<'a> {
        fn name(&self) -> &str { "TestRunnable" }
        fn id(&self) -> RunnableID { self.id }
        fn number_of_inputs(&self) -> usize { self.number_of_inputs }
        fn output_destinations(&self) -> &Vec<(&'static str, RunnableID, RunnableID)> { &self.destinations }
        fn implementation(&self) -> &Implementation { self.implementation }
        fn get_state(&self) -> &RunnableState { &self.state }

        fn init(&mut self) -> bool { false }
        fn write_input(&mut self, _input_number: usize, _new_value: JsonValue) {}
        fn input_full(&self, _input_number: usize) -> bool { true }
        fn inputs_ready(&self) -> bool { true }
        fn get_inputs(&mut self) -> Vec<Vec<JsonValue>> {
            vec!(vec!(serde_json::from_str("Input").unwrap()))
        }
        fn blocked_on_output(&self) -> bool { self.blocked_on_output }
        fn set_state(&mut self, new_state: RunnableState) { self.state = new_state }
    }

    fn test_run_list(runnables: Vec<&mut Runnable>) -> RunList {
        let (runset_tx, _runset_rx): (Sender<RunSet>, Receiver<RunSet>) = mpsc::channel();
        let (_output_tx, output_rx): (Sender<OutputSet>, Receiver<OutputSet>) = mpsc::channel();
        let mut run_list = RunList::new(runset_tx, output_rx, runnables);
        run_list.init();
        run_list
    }

    #[test]
    fn blocked_works() {
        let mut r0 = TestRunnable::new(0, 0, vec!(("", 1, 0), ("", 1, 0)));
        let mut r1 = TestRunnable::new(1, 1, vec!());
        let runnables = vec!(&mut r0 as &mut Runnable, &mut r1 as &mut Runnable);
        let mut run_list = test_run_list(runnables);

// Indicate that 0 is blocked by 1
        run_list.blocked_by(1, 0);
        assert!(run_list.is_blocked(0));
    }

    #[test]
    fn get_works() {
        let mut r0 = TestRunnable::new(0, 0, vec!(("", 1, 0), ("", 1, 0)));
        let mut r1 = TestRunnable::new(1, 1, vec!());
        let runnables = vec!(&mut r0 as &mut Runnable, &mut r1 as &mut Runnable);
        let mut run_list = &test_run_list(runnables);
        let got = run_list.get(1);
        assert_eq!(got.id(), 1)
    }

    #[test]
    fn no_next_if_none_ready() {
        let mut r0 = TestRunnable::new(0, 0, vec!(("", 1, 0), ("", 1, 0)));
        let mut r1 = TestRunnable::new(1, 1, vec!());
        let mut r2 = TestRunnable::new(2, 1, vec!());
        let runnables = vec!(&mut r0 as &mut Runnable, &mut r1 as &mut Runnable, &mut r2 as &mut Runnable);
        let mut _run_list = test_run_list(runnables);

// TODO peek into channel to see or try a get without timeout?
//        assert!(runs.next().is_none());
    }

    #[test]
    fn inputs_ready_makes_ready() {
        let mut r0 = TestRunnable::new(0, 0, vec!(("", 1, 0), ("", 1, 0)));
        let mut r1 = TestRunnable::new(1, 1, vec!());
        let mut r2 = TestRunnable::new(2, 1, vec!());
        let runnables = vec!(&mut r0 as &mut Runnable, &mut r1 as &mut Runnable, &mut r2 as &mut Runnable);
        let mut run_list = &test_run_list(runnables);

// Indicate that 0 has all it's inputs read
        run_list.inputs_ready(0);

// TODO get from channel?
//        assert_eq!(runs.next().unwrap(), 0);
    }

    #[test]
    fn blocked_is_not_ready() {
        let mut r0 = TestRunnable::new(0, 0, vec!(("", 1, 0), ("", 1, 0)));
        let mut r1 = TestRunnable::new(1, 1, vec!());
        let mut r2 = TestRunnable::new(2, 1, vec!());
        let runnables = vec!(&mut r0 as &mut Runnable, &mut r1 as &mut Runnable, &mut r2 as &mut Runnable);
        let mut run_list = test_run_list(runnables);

// Indicate that 0 is blocked by 1
        run_list.blocked_by(1, 0);

// Indicate that 0 has all it's inputs read
        run_list.inputs_ready(0);

// TODO peek into channel to see or try a get without timeout?
        /*
        match run_list.next() {
            None => assert!(true),
            Some(_) => assert!(false)
        }
        */
    }

    #[test]
    fn unblocking_makes_ready() {
        let mut r0 = TestRunnable::new(0, 0, vec!(("", 1, 0), ("", 1, 0)));
        let mut r1 = TestRunnable::new(1, 1, vec!());
        let mut r2 = TestRunnable::new(2, 1, vec!());
        let runnables = vec!(&mut r0 as &mut Runnable, &mut r1 as &mut Runnable, &mut r2 as &mut Runnable);
        let mut run_list = test_run_list(runnables);

// Indicate that 0 is blocked by 1
        run_list.blocked_by(1, 0);

// Indicate that 0 has all it's inputs read
        run_list.inputs_ready(0);

// TODO peek into channel to see or try a get without timeout?
//        assert_eq!(runs.next(), None);

// now unblock 0 by 1
        run_list.unblock_senders_to(1);

// Now runnable with id 0 should be ready and served up by next
// TODO peek into channel to see or try a get without timeout?
//        assert_eq!(runs.next(), Some(0));
    }

    #[test]
    fn unblocking_doubly_blocked_runnable_not_ready() {
        let mut r0 = TestRunnable::new(0, 0, vec!(("", 1, 0), ("", 1, 0)));
        let mut r1 = TestRunnable::new(1, 1, vec!());
        let mut r2 = TestRunnable::new(2, 1, vec!());
        let runnables: Vec<&mut Runnable> = vec!(&mut r0 as &mut Runnable, &mut r1 as &mut Runnable, &mut r2 as &mut Runnable);

        let run_list = &mut test_run_list(runnables);

        // Indicate that 0 is blocked by 1 and 2
        run_list.blocked_by(1, 0);
        run_list.blocked_by(2, 0);

        // Indicate that 0 has all it's inputs read
        run_list.inputs_ready(0);

        // Check that initially 0 is blocked and is not returned as next runnably
        assert_eq!(run_list.next_ready(), None);

        // now unblock 0 by 1
        run_list.unblock_senders_to(1);

        // Now runnable with id 0 should still not be ready as it is still blocked by 2
        assert_eq!(run_list.next_ready(), None);
    }
}
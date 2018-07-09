use runnable::Runnable;
use std::collections::HashSet;
use std::fmt;
use std::time::Instant;
use serde_json::Value as JsonValue;
use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use implementation::Implementation;
use runnable::RunnableID;

pub struct Metrics {
    num_runnables: usize,
    dispatches: u32,
    outputs_sent: u32,
    start_time: Instant,
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

    inputs_satisfied:
    A list of runnables who's inputs are satisfied.

    blocking:
    A list of tuples of runnable ids where first id is id of the runnable data is trying to be sent
    to, and the second id is the id of the runnable trying to send data.

    ready:
    A list of Runnables who are ready to be run, they have their inputs satisfied and they are not
    blocked on the output (so their output can be produced).
*/
pub struct RunList<'a> {
    runnables: Vec<&'a mut Runnable>,
    can_run: HashSet<RunnableID>,
    blocking: Vec<(RunnableID, RunnableID)>,
    // blocking_id, blocked_id
    metrics: Metrics,
    sender: Sender<RunSet<'a>>,
    receiver: Receiver<OutputSet>
}

// Set of information needed to run an implementation
pub struct RunSet<'a> {
    pub id: RunnableID,
    pub implementation: &'a Implementation,
    pub inputs: Vec<Vec<JsonValue>>
}

// Set of information output by an Implementation
pub struct OutputSet {
    from: RunnableID,
    output: JsonValue
}

impl<'a> RunList<'a> {
    fn new(sender: Sender<RunSet<'a>>, receiver: Receiver<OutputSet>, runnables: Vec<&'a mut Runnable>) -> Self {
        let len = runnables.len();
        RunList {
            runnables,
            can_run: HashSet::<RunnableID>::new(),
            blocking: Vec::<(RunnableID, RunnableID)>::new(),
            metrics: Metrics::new(len),
            sender,
            receiver
        }
    }

    /*
        This initializes each runnable in the list by calling it's `init` method.
        The `init` method returns a boolean to indicate that it's inputs are fulfilled (or not).
        This information is added to the RunList to control the readyness of  the Runnable to be executed.
    */
    fn init(&'a mut self) {
        debug!("Initializing runnables in the runlist");
        for mut runnable in &mut self.runnables {
            debug!("\tInitializing runnable #{} '{}'", &runnable.id(), runnable.name());
            if runnable.init() {
                self.inputs_ready(runnable.id());
            }
        }
    }

    /*
        Print out a simple debug of the various lists in the runlist
    */
    fn debug(&self) {
        debug!("Dispatch count: {}", self.metrics.dispatches);
        debug!("       Can Run: {:?}", self.can_run);
        debug!("      Blocking: {:?}", self.blocking);
        debug!("-------------------------------------");
    }

    /*
        Call this when a flow ends to print out metrics related to it's execution
    */
    fn flow_end(&self) {
        self.debug();
        debug!("Metrics: \n {}", self.metrics);
    }

    fn get(&'a mut self, id: RunnableID) -> &'a Runnable {
        self.runnables[id]
    }

    fn done(&'a mut self, id: RunnableID, run_again: bool) {
        // if it wants to run again and it can (inputs ready) then add back to the Can Run list
        if run_again && self.get(id).inputs_ready() {
            self.inputs_ready(id);
        }
    }

    /*
        Save the fact that a particular Runnable's inputs are now satisfied and so it maybe ready
        to run (if not blocked sending on it's output)
    */
    fn inputs_ready(&'a mut self, id: RunnableID) {
        debug!("\t\t\tRunnable #{} inputs are ready", id);
        self.can_run.insert(id);

        if !self.is_blocked(id) {
            self.dispatch(id);
        }
    }

    /*
        When a runnable consumes it's inputs, then take if off the list of runnables with inputs ready
        until new inputs are sent later and it is added again
    */
    fn inputs_consumed(&mut self, id: RunnableID) {
        debug!("\tRunnable #{} consumed its inputs, removing from the 'Can Run' list", id);
        self.can_run.remove(&id);
    }

    // Save the fact that the runnable 'blocked_id' is blocked on it's output by 'blocking_id'
    fn blocked_by(&mut self, blocking_id: RunnableID, blocked_id: RunnableID) {
        // avoid deadlocks by a runnable blocking itself
        if blocked_id != blocking_id {
            debug!("\t\t\tRunnable #{} is now blocked on output by Runnable #{}", &blocked_id, &blocking_id);
            self.blocking.push((blocking_id, blocked_id));
        }
    }

    // unblock all runnables that were blocked trying to send to blocker_id by removing all entries
    // in the list where the first value (blocking_id) matches the destination_id
    // when each is unblocked on output, if it's inputs are satisfied, then it is ready to be run
    // again, so put it on the ready queue
    fn unblock_senders_to(&'a mut self, blocker_id: RunnableID) {
        if !self.blocking.is_empty() {
            let mut unblocked_list: Vec<RunnableID> = vec!();

            for &(blocking_id, blocked_id) in &self.blocking {
                if blocking_id == blocker_id {
                    debug!("\t\tRunnable #{} <-- #{} - block removed", blocking_id, blocked_id);
                    unblocked_list.push(blocked_id);
                }
            }

            // when done remove all entries from the blocking list where it was this blocker_id
            self.blocking.retain(|&(blocking_id, _blocked_id)| blocking_id != blocker_id);

            // see if any of the runnables unblocked should be dispatched
            // (they could be blocked on others not the one that unblocked)
            for unblocked in unblocked_list {
                if self.can_run.contains(&unblocked) && !self.is_blocked(unblocked) {
                    self.dispatch(unblocked);
                }
            }
        }
    }

    /*
        A runnable has become able to be run, run the implementation on it's input data
            - gather it's inputs (marking them as consumed)
            - unblock others attempting to send to it
            - get the implementation to run on those inputs
            - send the id, implementation and inputs on the channel for dispatching
    */
    fn dispatch(&'a mut self, id: RunnableID) {
        let mut runnable = self.get(id);
        let inputs = runnable.get_inputs();
        self.inputs_consumed(id);
        self.unblock_senders_to(id);

        let implementation = runnable.implementation();

        debug!("Runnable #{} '{}' dispatched with inputs: {:?}", id, runnable.name(), inputs);
        self.metrics.dispatches += 1;
        self.sender.send(RunSet{id, implementation, inputs});
    }

    // TODO ADM optimize this by also having a flag in the runnable?
    // Or use the blocked_id as a key to a HashSet?
    // See if there is any tuple in the vector where the second (blocked_id) is the one we're after
    fn is_blocked(&self, id: RunnableID) -> bool {
        for &(_blocking_id, blocked_id) in &self.blocking {
            if blocked_id == id {
                return true;
            }
        }
        false
    }
}

/*
    For now the server will only execute one RunList in one thread, locally
*/
pub struct RunListServer<'a> {
    sender: Sender<RunSet<'a>>,
    receiver: Receiver<OutputSet>,
}

impl<'a> RunListServer<'a> {
    /*
        This function:
        - creates a new runlist with the vector of runnables
        - initialized each runnable
        - starts a RunList server thread that loops infinitely receiving outputs sent from runnables
        implementations, then processing those outputs.

        This function must correctly prepare the run_list before starting the server (on another thread)
        to serve RunSets.
    */
    pub fn connect(runnables: Vec<&'a mut Runnable>) -> (Receiver<RunSet>,
                                                      Sender<OutputSet>) {
        let (runset_tx, runset_rx): (Sender<RunSet<'a>>, Receiver<RunSet<'a>>) = mpsc::channel();
        let (output_tx, output_rx): (Sender<OutputSet>, Receiver<OutputSet>) = mpsc::channel();
        let mut run_list = RunList::new(runset_tx, output_rx, runnables);
        run_list.init();

        thread::spawn(move || Self::receiver_loop(run_list) );

        (runset_rx, output_tx)
    }

    /*
        Loop forever receiving OutputSets from running implementations and processing them
    */
    fn receiver_loop(mut run_list: RunList<'a>) {
        loop {
            let output_set = run_list.receiver.recv().unwrap();
            Self::process_output_set(&mut run_list, output_set);
        }
    }

    /*
        Then take the output and send it to all destination IOs on different runnables it should be
        sent to, marking the source runnable as blocked because those others must consume the output
        if those other runnables have all their inputs, then mark them accordingly.
    */
    fn process_output_set(run_list: &'a mut  RunList<'a>, output_set: OutputSet) {
        let runnable = run_list.get(output_set.from);

        for &(output_route, destination_id, io_number) in runnable.output_destinations() {
            let mut destination = run_list.get(destination_id);
            let output_value = output_set.output.pointer(output_route).unwrap();
            debug!("\t\tRunnable #{} '{}{}' sending output '{}' to Runnable #{} '{}' input #{}",
                   runnable.id(), runnable.name(), output_route, output_value, &destination_id,
                   destination.name(), &io_number);
            destination.write_input(io_number, output_value.clone());
            run_list.metrics.outputs_sent += 1;
            if destination.input_full(io_number) {
                run_list.blocked_by(destination_id, output_set.from);
            }

            if destination.inputs_ready() {
                run_list.inputs_ready(destination_id);
            }
        }

        // TODO consider special output to signal end?
        // Maybe when running as a thread just detect death of thread and print this then
        //debug!("\tRunnable #{} '{}' completed", id, runnable.name());
    }
}

#[cfg(test)]
mod tests {
    use super::{RunList, RunSet};
    use super::Runnable;
    use serde_json;
    use serde_json::Value as JsonValue;
    use super::super::implementation::{Implementation, RunAgain, RUN_AGAIN};
    use super::OutputSet;
    use super::RunnableID;
    use std::sync::mpsc::{Sender, Receiver};
    use std::sync::mpsc;

    struct TestImplementation;

    impl Implementation for TestImplementation {
        fn run(&self, id: RunnableID, inputs: Vec<Vec<JsonValue>>, tx: Sender<OutputSet>) -> RunAgain {
            // get the input value
            let input = inputs.get(0).unwrap().get(0).unwrap().clone();

            // Send the input directly as the output
            self.send_output(id, &tx, input);
            RUN_AGAIN
        }
    }

    struct TestRunnable<'a> {
        id: RunnableID,
        number_of_inputs: usize,
        destinations: Vec<(&'static str, RunnableID, usize)>,
        implementation: &'a Implementation,
    }

    impl<'a> TestRunnable<'a> {
        fn new(id: RunnableID, number_of_inputs: usize, destinations: Vec<(&'static str, RunnableID, usize)>) -> TestRunnable {
            TestRunnable {
                id,
                number_of_inputs,
                destinations,
                implementation: &TestImplementation,
            }
        }
    }

    impl<'a> Runnable for TestRunnable<'a> {
        fn name(&self) -> &str { "TestRunnable" }
        fn number_of_inputs(&self) -> usize { self.number_of_inputs }
        fn id(&self) -> RunnableID { self.id }
        fn init(&mut self) -> bool { false }
        fn write_input(&mut self, _input_number: usize, _new_value: JsonValue) {}
        fn input_full(&self, _input_number: usize) -> bool { true }
        fn inputs_ready(&self) -> bool { true }
        fn get_inputs(&mut self) -> Vec<Vec<JsonValue>> {
            vec!(vec!(serde_json::from_str("Input").unwrap()))
        }
        fn output_destinations(&self) -> &Vec<(&'static str, RunnableID, RunnableID)> { &self.destinations }
        fn implementation(&self) -> &Implementation { self.implementation }
    }

    fn test_run_list(runnables: Vec<&mut Runnable>) -> RunList {
        let (runset_tx, runset_rx): (Sender<RunSet>, Receiver<RunSet>) = mpsc::channel();
        let (output_tx, output_rx): (Sender<OutputSet>, Receiver<OutputSet>) = mpsc::channel();
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
        let mut run_list = test_run_list(runnables);
        let got = run_list.get(1);
        assert_eq!(got.id(), 1)
    }

    #[test]
    fn no_next_if_none_ready() {
        let mut r0 = TestRunnable::new(0, 0, vec!(("", 1, 0), ("", 1, 0)));
        let mut r1 = TestRunnable::new(1, 1, vec!());
        let mut r2 = TestRunnable::new(2, 1, vec!());
        let runnables = vec!(&mut r0 as &mut Runnable, &mut r1 as &mut Runnable, &mut r2 as &mut Runnable);
        let mut run_list = test_run_list(runnables);

        // TODO peek into channel to see or try a get without timeout?
//        assert!(runs.next().is_none());
    }

    #[test]
    fn inputs_ready_makes_ready() {
        let mut r0 = TestRunnable::new(0, 0, vec!(("", 1, 0), ("", 1, 0)));
        let mut r1 = TestRunnable::new(1, 1, vec!());
        let mut r2 = TestRunnable::new(2, 1, vec!());
        let runnables = vec!(&mut r0 as &mut Runnable, &mut r1 as &mut Runnable, &mut r2 as &mut Runnable);
        let mut run_list = test_run_list(runnables);

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

        let mut run_list = test_run_list(runnables);

        // Indicate that 0 is blocked by 1 and 2
        run_list.blocked_by(1, 0);
        run_list.blocked_by(2, 0);

        // Indicate that 0 has all it's inputs read
        run_list.inputs_ready(0);

        // TODO peek into channel to see or try a get without timeout?
//        assert_eq!(run_list.next(), None);

        // now unblock 0 by 1
        run_list.unblock_senders_to(1);

        // Now runnable with id 0 should still not be ready as still blocked on 2
        // TODO peek into channel to see or try a get without timeout?
//        assert_eq!(run_list.next(), None);
    }
}
use runnable::Runnable;
use std::fmt;
use std::time::Instant;
use serde_json::Value as JsonValue;
use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use implementation::Implementation;
use runnable::RunnableID;
use runlist::{RunSet, OutputSet, RunList};
use executor::executor;

pub struct RunListServer<'a> {
    // A channel that is used to send RunSets for running implementations
    runset_tx: Sender<RunSet<'a>>,
    // A channel for receiving back OutputSets from running implementations
    output_rx: Receiver<OutputSet>,
    // A channel used by executors to send output sets to this RunListServer
    output_tx: Sender<OutputSet>,
    // A channel used by executors to get RunSets from this RunListServer
    runset_rx: Receiver<RunSet<'a>>,
    // A channel for clients to send run_lists for execution on
    runlist_tx: Sender<RunList>,
    // A channel for the receiver_loop to receive runlists for execution on
    runlist_rx: Receiver<RunList>
}

impl<'a> RunListServer<'a> {
    /// Clients should call this to connect to the runlist server
    ///
    /// # Example
    /// ```
    /// use flowrlib::runlist::RunListServer;
    /// use std::process::exit;
    ///
    /// let rls = RunListServer::connect();
    ///
    /// exit(0);
    /// ```
    /*
        Create a RunListServer - create channels to be used to send RunSets to executors to be
        executed and get OutputSets back from run implementations to modify the RunList
    */
    pub fn connect() -> Self {
        let (runset_tx, runset_rx): (Sender<RunSet<'a>>, Receiver<RunSet<'a>>) = mpsc::channel();
        let (output_tx, output_rx): (Sender<OutputSet>, Receiver<OutputSet>) = mpsc::channel();
        let (runlist_tx, runlist_rx): (Sender<RunList>, Receiver<RunList>) = mpsc::channel();
        let rls = RunListServer {
            runset_tx,
            output_rx,
            output_tx,
            runset_rx,
            runlist_tx,
            runlist_rx
        };

        // TODO in the future this might discover and connect to an existing server....
        // Start a receiver thread for receiving RunLists submitted to the server
        thread::spawn(move || Self::runlist_receiver(submission_rx));

        // Return the RunListServer to use to submit sets of runnables for execution
        rls
    }

    /// Called by clients to execute a set of runnables
    ///
    /// # Example
    /// ```
    /// use flowrlib::runlist::RunListServer;
    /// use std::process::exit;
    ///
    /// let rls = RunListServer::connect();
    /// rls.submit(get_runnables());
    ///
    /// exit(0);
    /// ```
    /*
        Create a new runlist with the vector of runnables,
        sends runlist to RunListServer for dispatching to executors,
        run an executor locally to handle stdio locally
    */
    pub fn execute(&mut self, runnables: Vec<&'a mut Runnable>) {
        let mut run_list = RunList::new(self.runset_tx, self.output_rx, runnables);
        self.submission_tx.send(run_list);

        // have the client execute the execution loop also to send/receive stdio locally
        executor(self.runset_rx, self.output_tx);
    }

    /*
        Loop forever receiving RunLists for execution
    */
    fn runlist_receiver(rx: Receiver<RunList>) {
        // TODO for now execute one runlist then exit
//        loop {
            // Start execution thread(s) locally - maybe one per core
        // For now, just let the client execute the execution_loop
//            thread::spawn(move || execution_loop(runset_rx, output_tx));

            // TODO find other executors elsewhere

            // wait to receive a runlist
            let run_list = rx.receiver.recv().unwrap();

            // then initialize it
            run_list.init();

        // TODO to execute more than one runlist in parallel this would need to spawn a thread
            receiver_loop(run_list);
//        }
    }

    /*
        Loop forever receiving OutputSets from running implementations and processing them
        // TODO separate dispatching and processing output sets coming back
    */
    fn receiver_loop(mut run_list: RunList<'a>) {
        // TODO keep track of how many implementations are running so we know when we can die?

        // Get next runnable ready to be dispatched
        while let runnableId = run_list.ready() {
            // Dispatch it to be run on an executor
            run_list.dispatch(runnableId);

            // TODO for now we'll assume it will only return one output_set
            // receive back output sets from the running runnable
            let output_set = run_list.receiver.recv().unwrap();

            // change state of run_list based on the output set
            run_list.process_output_set(output_set);
        }
    }
}
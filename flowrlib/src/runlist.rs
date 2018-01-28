use runnable::Runnable;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;

/*
    inputs_satisfied:
    A list of runnables who's inputs are satisfied.

    blocked:
    A list of tuples of runnable ids where first runnable_id is where data is trying
    to be sent to, and the second runnable_id is the runnable trying to send to it.
    Vec<(runnable_to_send_to, runnable_that_is_blocked_on_output)>

    Note that a runnable maybe blocking multiple others trying to send to it.
    Those others maybe blocked trying to send to multiple.
    So, when a runnable is run, we remove all entries that depend on it.
*/
pub struct RunList {
    runnables: Vec<Arc<Mutex<Runnable>>>,
    inputs_satisfied: HashSet<usize>,
    blocking: Vec<(usize, usize)>,
    ready: Vec<usize>
}

impl RunList {
    pub fn new() -> Self {
        RunList {
            runnables: Vec::<Arc<Mutex<Runnable>>>::new(),
            inputs_satisfied: HashSet::<usize>::new(),
            blocking: Vec::<(usize, usize)>::new(),
            ready: Vec::<usize>::new(),
        }
    }

    pub fn set_runnables(&mut self, runnables: Vec<Arc<Mutex<Runnable>>>) {
        self.runnables = runnables;
    }

    // Get a runnable from the runnable ID
    pub fn get(&self, id: usize) -> Arc<Mutex<Runnable>> {
        self.runnables[id].clone()
    }

    // save the fact that a particular Runnable's inputs are now satisfied
    pub fn inputs_ready(&mut self, id: usize) {
        info!("Runnable #{}'s inputs are all ready", id);

        if self.is_blocked(id) {
            self.inputs_satisfied.insert(id);
        } else {
            info!("Marking #{} as ready", id);
            self.ready.push(id);
        }
    }

    // Return the next runnable at the head of the ready list if there is one
    pub fn next(&mut self) -> Option<Arc<Mutex<Runnable>>> {
        if self.ready.len() == 0 {
            return None;
        }

        info!("Ready list: {:?}", self.ready);

        // get the ID of the next runnable to be run
        let id = self.ready.remove(0);
        Some(self.runnables[id].clone())
    }

    // Save the fact that the runnable 'blocked_id' is blocked on it's output by 'blocking_id'
    pub fn blocked_by(&mut self, blocking_id: usize, blocked_id: usize) {
        info!("Runnable #{} is blocking runnable #{}", &blocking_id, &blocked_id);
        self.blocking.push((blocking_id, blocked_id));
    }

    // unblock all runnables that were blocked trying to send to destination_id by removing all entries
    // in the list where the first value (blocking_id) matches the destination_id
    pub fn unblock_by(&mut self, destination_id: usize) {
        info!("Unblocking runnables blocked on #{}", &destination_id);
        for &(blocking_id, blocked_id) in &self.blocking {
            if blocking_id == destination_id {
                if self.inputs_satisfied.remove(&blocked_id) {
                    self.ready.push(blocked_id);
                }
            }
        }

        self.blocking.retain(|&(blocking_id, _blocked_id)| blocking_id != destination_id);
    }

    // See if there is any tuple in the vector where the second (blocked_id) is the one we're after
    fn is_blocked(&self, id: usize) -> bool {
        for &(_blocking_id, blocked_id) in &self.blocking {
            if blocked_id == id {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::RunList;
    use super::Runnable;
    use std::sync::{Arc, Mutex};

    struct TestRunnable {
        id: usize
    }

    impl TestRunnable {
        fn new(id: usize) -> TestRunnable {
            TestRunnable{id}
        }
    }

    impl Runnable for TestRunnable {
        fn init(&mut self) -> bool { false }
        fn write_input(&mut self, _input_number: usize, _new_value: Option<String>) {}
        fn inputs_satisfied(&self) -> bool {false}
        fn run(&mut self) -> Option<String> { Some("Output".to_string())}
        fn output_destinations(&self) -> Vec<(usize, usize)> {vec!((1,0))}
        fn id(&self) -> usize { self.id }
        fn to_code(&self) -> String {"fake code".to_string()}
    }

    fn test_runnables() -> RunList {
        let mut runs = RunList::new();
        let r0 = Arc::new(Mutex::new(TestRunnable::new(0)));
        let r1 = Arc::new(Mutex::new(TestRunnable::new(1)));
        runs.set_runnables(vec!(r0, r1));
        runs
    }

    #[test]
    fn get_works() {
        let runs = test_runnables();
        let got_arc = runs.get(1);
        let got = got_arc.lock().unwrap();
        assert_eq!(got.id(), 1)
    }

    #[test]
    fn blocked_works() {
        let mut runs = test_runnables();

        // Indicate that 0 is blocked by 1
        runs.blocked_by(1, 0);
        assert!(runs.is_blocked(0));
    }

    #[test]
    fn no_next_if_none_ready() {
        let mut runs = test_runnables();

        assert!(runs.next().is_none());
    }

    #[test]
    fn inputs_ready_makes_ready() {
        let mut runs = test_runnables();

        // Indicate that 0 has all it's inputs read
        runs.inputs_ready( 0);

        match runs.next() {
            None => assert!(false),
            Some(arc) => {
                let next = arc.lock().unwrap();
                assert_eq!(next.id(), 0);
            }
        }
    }

    #[test]
    fn blocked_is_not_ready() {
        let mut runs = test_runnables();

        // Indicate that 0 is blocked by 1
        runs.blocked_by(1, 0);

        // Indicate that 0 has all it's inputs read
        runs.inputs_ready( 0);

        match runs.next() {
            None => assert!(true),
            Some(_) => assert!(false)
        }
    }

    #[test]
    fn unblocking_makes_ready() {
        let mut runs = test_runnables();

        // Indicate that 0 is blocked by 1
        runs.blocked_by(1, 0);

        // Indicate that 0 has all it's inputs read
        runs.inputs_ready( 0);

        assert!(runs.next().is_none());

        // now unblock 0 by 1
        runs.unblock_by(1);

        match runs.next() {
            None => assert!(false),
            Some(arc) => {
                let next = arc.lock().unwrap();
                assert_eq!(next.id(), 0);
            }
        }
    }
}
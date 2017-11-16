use description::name::Name;
use description::datatype::DataType;
use description::io::{Output, Input, InputOutput, OutputInput};

/*
	Unidirectional connection between an Output and an Input
	Can only carry a single datatype
 */
pub struct Connection<'a> {
	name: Name,
	data_type: DataType<'a>,
// change to references to ensure they refer to real Input/Output and not copy them
	from: Output,
	to: Input,
}

/*
impl Connection {
	// TODO change to references later
	fn new(name: Name, data_type: DataType, from: Output, to: Input) -> Connection {
		Connection {
			name: name,
			data_type: data_type,
			from: from,
			to : to,
		}
	}

	fn validate_fields(&self) -> parser::Result {
		self.name.validate_fields("Connection") // TODO early return

		// Validate other fields exist and are valid syntax
	}
}

*/

/*
	Bidirectional request from one IO to another with a datatype for the
	request and another datatype for the response.
 */
pub struct Request<'a>  {
	name: Name,
	// change to references to ensure they refer to real Input/Output and not copy them
	from: OutputInput,
	request_data_type: DataType<'a>,
	to: InputOutput,
	response_data_type: DataType<'a>,
}

/*
impl Request {
	fn validate_fields(&self) -> parser::Result {
		self.name.validate_fields("Result") // TODO early return

		// Validate other fields exist and are valid syntax
	}
}
*/

pub struct ConnectionSet<'a> {
	connections: Vec<Connection<'a>>,
	requests: Vec<Request<'a>>,
}

/*
From the Connection set, return a subset that are connected to a specific name
 */
/*
impl ConnectionSet {
	pub fn new(connections: Vec<Connection>, requests: Vec<Request>) -> ConnectionSet {
		ConnectionSet {
			connections: connections,
			requests: requests,
		}
	}

	pub fn get_subset(&self, name: &String) -> ConnectionSet {
		// TODO
		ConnectionSet::new(vec![], vec![])
	}

	pub fn validate_fields(&self) -> parser::Result  {
		for connection in &self.connections {
			connection.validate_fields(); // TODO early return
		}
		for request in &self.requests {
			request.validate_fields(); // TODO early return
		}

		parser::Result::Valid
	}

	pub fn check(connection_set: &ConnectionSet, io_sets: &Vec<&IOSet>, values: &Vec<Value>)
    -> parser::Result {
		// TODO
		// for each connection
		// connected at both ends to something passed in, directions and types match
		// 		validateConnection in itself, not to subflow
		parser::Result::Valid
	}
}
*/
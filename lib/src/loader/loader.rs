use std::path::PathBuf;

use description::flow::Flow;
use description::function::Function;
use loader::file_helper::get_contents;
use loader::file_helper::get_canonical_path;
use loader::loader_helper::get_loader;

pub trait Loader {
    fn load_flow(&self, contents: &str) -> Result<Flow, String>;
    fn load_function(&self, contents: &str) -> Result<Function, String>;
}

pub trait Validate {
    fn validate(&self) -> Result<(), String>;
}

/// # Example
/// ```
/// use std::path::PathBuf;
/// use flowlib::loader::loader;
///
/// let path = PathBuf::from("../samples/hello-world-simple-toml/context.toml");
/// loader::load_flow(path).unwrap();
/// ```
pub fn load_flow(file_path: PathBuf) -> Result<Flow, String> {
    let loader = get_loader(&file_path)?;
    let contents = get_contents(&file_path)?;
    let mut flow = loader.load_flow(&contents)?;
    flow.source = file_path;
    flow.validate()?;
    load_functions(&mut flow)?;
    load_subflows(&mut flow)?;
    Ok(flow)
}

/// # Example
/// ```
/// use std::path::PathBuf;
/// use flowlib::loader::loader;
///
/// let path = PathBuf::from("../samples/hello-world-simple-toml/terminal.toml");
/// loader::load_function(&path).unwrap();
/// ```
pub fn load_function(file_path: &PathBuf) -> Result<Function, String> {
    let loader = get_loader(file_path)?;
    let contents = get_contents(file_path)?;
    let function = loader.load_function(&contents)?;
    function.validate()?;
    Ok(function)
}

/*
    Load all functions references from a flow
*/
fn load_functions(flow: &mut Flow) -> Result<(), String> {
    if let Some(ref mut function_refs) = flow.function {
        for ref mut function_ref in function_refs {
            let function_path = get_canonical_path(PathBuf::from(&flow.source),
                                                   PathBuf::from(&function_ref.source));
            function_ref.function = load_function(&function_path)?;
        }
    }
    Ok(())
}

/*
    Load all flows references from a flow
*/
fn load_subflows(flow: &mut Flow) -> Result<(), String> {
    // Load subflows from References
    if let Some(ref mut flow_refs) = flow.flow {
        for ref mut flow_ref in flow_refs {
            let subflow_path = get_canonical_path(PathBuf::from(&flow.source),
                                                  PathBuf::from(&flow_ref.source));
            let subflow = load_flow(subflow_path)?;
            flow_ref.flow = subflow;
        }
    }
    Ok(())
}
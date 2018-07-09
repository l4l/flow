use std::io::Result;
use strfmt::strfmt;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::collections::HashSet;
use std::io::{Error, ErrorKind};
use generator::code_gen::CodeGenTables;
use model::runnable::Runnable;

const RUNNABLES_PREFIX: &'static str = "
// Flow Run-time library references
use flowrlib::runnable::Runnable;
use flowrlib::implementation::Implementation;
{value_used}
{function_used}\n";

const GET_RUNNABLES: &'static str = "
pub fn get_runnables() -> Vec<&Runnable> {{
    let mut runnables = Vec::<&Runnable>::with_capacity({num_runnables});\n\n";

const RUNNABLES_SUFFIX: &'static str = "
    runnables
}";

// Create the 'runnables.rs' file in the output project's source folder
pub fn create(src_dir: &PathBuf, tables: &CodeGenTables) -> Result<()> {
    let mut file = src_dir.clone();
    file.push("runnables.rs");
    let mut runnables_rs = File::create(&file)?;
    let contents = contents(tables,implementations(&tables)?);
    runnables_rs.write_all(contents.unwrap().as_bytes())
}

fn uses_value(runnables: &Vec<Box<Runnable>>) -> bool {
    for runnable in runnables {
        if runnable.get_type() == "Value" {
            return true;
        }
    }

    false
}

fn uses_function(runnables: &Vec<Box<Runnable>>) -> bool {
    for runnable in runnables {
        if runnable.get_type() == "Function" {
            return true;
        }
    }
    return false;
}

fn contents(tables: &CodeGenTables, implementations: (Vec<String>, Vec<String>)) -> Result<String> {
    let mut vars = HashMap::new();

    if uses_value(&tables.runnables) {
        vars.insert("value_used".to_string(), "use flowrlib::value::Value;");
    } else {
        vars.insert("value_used".to_string(), "");
    }

    if uses_function(&tables.runnables) {
        vars.insert("function_used".to_string(), "use flowrlib::function::Function;");
    } else {
        vars.insert("function_used".to_string(), "");
    }

    let mut content = strfmt(RUNNABLES_PREFIX, &vars).unwrap();

    content.push_str("\n// Implementations used\n");
    for implementation_use in implementations.0 {
        content.push_str(&implementation_use);
    }

    content.push_str("\n\n// Implementations\n");
    for implementation_instantiation in implementations.1 {
        content.push_str(&implementation_instantiation);
    }

    content.push_str(&runnables(tables));

    Ok(content)
}

/*
    Generate the string contents that declares an array of runnables
*/
fn runnables(tables: &CodeGenTables) -> String {
    let mut runnables_declarations = String::new();
    let num_runnables = &tables.runnables.len().to_string();

    // add declaration of runnables array  - parameterized by the number of runnables
    let mut vars = HashMap::<String, &str>::new();
    vars.insert("num_runnables".to_string(), num_runnables);
    runnables_declarations.push_str(&strfmt(GET_RUNNABLES, &vars).unwrap());

    // Generate code for each of the runnables
    for runnable in &tables.runnables {
        let run_str = format!("    runnables.push(&{});\n",
                              runnable_to_code(runnable));
        runnables_declarations.push_str(&run_str);
    }

    runnables_declarations.push_str(RUNNABLES_SUFFIX);

    // return the string declaring runnables array
    runnables_declarations
}

/*
    Convert a set of references used flows in '/' format into use statements of rust
*/
fn implementations(tables: &CodeGenTables) -> Result<(Vec<String>, Vec<String>)> {
    let mut implementations_used: Vec<String> = Vec::new();
    let mut implementation_instantiations: Vec<String> = Vec::new();

    for lib_ref in &tables.lib_references {
        let lib_use = str::replace(&lib_ref, "/", "::");
        implementations_used.push(format!("use {};\n", lib_use));
        let parts = lib_ref.split('/').collect::<Vec<&str>>();
        let implementation = parts.last().unwrap();
        implementation_instantiations.push(format!("static {}: &Implementation = &{}{{}} as &Implementation;\n",
                                                   implementation.to_uppercase(), implementation));
    }

    // If Value is used then add a reference to an implementation of it from the std library
    if uses_value(&tables.runnables) {
        implementations_used.push("use flowstdlib::zero_fifo::Fifo;".to_string());

        implementation_instantiations.push(format!("static FIFO: &Implementation = &Fifo{{}} as &Implementation;\n"));
    }

    // Find all the functions that are not loaded from libraries
    let mut uses_declared = HashSet::new();
    for runnable in &tables.runnables {
        if let Some(source_url) = runnable.source_url() {
            let source = source_url.to_file_path()
                .map_err(|_e| Error::new(ErrorKind::InvalidData, "Could not convert to file path"))?;
            let usage = source.file_stem().unwrap();
            let use_string = format!("use {}::{};\n", usage.to_str().unwrap(), runnable.name());
            // Don't add the same use twice
            if uses_declared.insert(use_string.clone()) {
                implementations_used.push(use_string);
                implementation_instantiations.push(format!("static {}: &Implementation = &{} as &Implementation;\n",
                                                           runnable.get_implementation().to_uppercase(), runnable.get_implementation()));
            }
        }
    }

    Ok((implementations_used, implementation_instantiations))
}

// Output a statement that instantiates an instance of the Runnable type used, that can be used
// to build the list of runnables
fn runnable_to_code(runnable: &Box<Runnable>) -> String {
    let mut code = format!("{}::new(\"{}\", ", runnable.get_type(), runnable.alias());
    match &runnable.get_inputs() {
        // No inputs, so put a '0' and an empty vector of input depths
        &None => code.push_str(&format!("{}, {}, vec!(), ", 0, runnable.is_static_value())),

        // Some inputs, so put the number and the vector of input depths
        Some(inputs) => {
            code.push_str(&format!("{}, {}, vec!(", inputs.len(), runnable.is_static_value()));
            for input in inputs {
                code.push_str(&format!("{}, ", input.depth()));
            }
            code.push_str(&format!("), "));
        }
    }
    code.push_str(&format!("{}, {}, ", runnable.get_id(),
                           runnable.get_implementation().to_uppercase()));

    code.push_str(&format!("{},", match runnable.get_initial_value() {
        None => "None".to_string(),
        Some(value) => format!("Some(json!({}))", value.to_string())
    }));

    // Add tuples of this function's output routes to runnables and the input it's connected to
    code.push_str(" vec!(");
    debug!("Runnable '{}' output routes: {:?}", runnable.name(), runnable.get_output_routes());
    for ref route in runnable.get_output_routes() {
        if route.0.is_empty() {
            code.push_str(&format!("(\"\", {}, {}),", route.1, route.2)); // no leading '/'
        } else {
            code.push_str(&format!("(\"/{}\", {}, {}),", route.0, route.1, route.2));
        }
    }
    code.push_str(")");

    code.push_str(")");

    code
}

#[cfg(test)]
mod test {
    use serde_json::Value as JsonValue;
    use model::value::Value;
    use model::io::IO;
    use model::function::Function;
    use model::runnable::Runnable;
    use url::Url;
    use super::runnable_to_code;

    #[test]
    fn test_value_to_code() {
        let value = Value::new("value".to_string(),
                               "String".to_string(),
                               Some(JsonValue::String("Hello-World".to_string())),
                               false,
                               "/flow0/value".to_string(),
                               Some(vec!(IO::new(&"Json".to_string(), &"".to_string()))),
                               vec!(("".to_string(), 1, 0)),
                               1);

        let br = Box::new(value) as Box<Runnable>;
        let code = runnable_to_code(&br);
        assert_eq!(code, "Value::new(\"value\", 1, false, vec!(1, ), 1, FIFO, Some(json!(\"Hello-World\")), vec!((\"\", 1, 0),))")
    }

    #[test]
    fn test_constant_value_to_code() {
        let value = Value::new(
            "value".to_string(),
            "String".to_string(),
            Some(JsonValue::String("Hello-World".to_string())),
            true,
            "/flow0/value".to_string(),
            Some(vec!(IO::new(&"Json".to_string(), &"".to_string()))),
            vec!(("".to_string(), 1, 0)),
            1);

        let br = Box::new(value) as Box<Runnable>;
        let code = runnable_to_code(&br);
        assert_eq!(code, "Value::new(\"value\", 1, true, vec!(1, ), 1, FIFO, Some(json!(\"Hello-World\")), vec!((\"\", 1, 0),))")
    }

    #[test]
    fn value_with_sub_route_output_to_code() {
        let value = Value::new(
            "value".to_string(),
            "String".to_string(),
            Some(JsonValue::String("Hello-World".to_string())),
            false,
            "/flow0/value".to_string(),
            Some(vec!(
                IO::new(&"Json".to_string(), &"".to_string()),
                IO::new(&"String".to_string(), &"".to_string()))),
            vec!(("".to_string(), 1, 0), ("sub_route".to_string(), 2, 0)),
            1);

        let br = Box::new(value) as Box<Runnable>;
        let code = runnable_to_code(&br);
        assert_eq!(code, "Value::new(\"value\", 1, false, vec!(1, ), 1, FIFO, Some(json!(\"Hello-World\")), vec!((\"\", 1, 0),(\"/sub_route\", 2, 0),))")
    }

    #[test]
    fn function_with_sub_route_output_to_code() {
        let function = Function::new(
            "Stdout".to_string(),
            "print".to_string(),
            Some(vec!()),
            Some(vec!(
                IO::new(&"Json".to_string(), &"".to_string()),
                IO::new(&"String".to_string(), &"".to_string())
            )),
            Url::parse("file:///fake/file").unwrap(),
            "/flow0/stdout".to_string(),
            None,
            vec!(("".to_string(), 1, 0), ("sub_route".to_string(), 2, 0)),
            0);

        let br = Box::new(function) as Box<Runnable>;
        let code = runnable_to_code(&br);
        assert_eq!(code, "Function::new(\"print\", 0, false, vec!(), 0, STDOUT, None, vec!((\"\", 1, 0),(\"/sub_route\", 2, 0),))")
    }

    #[test]
    fn function_to_code() {
        let function = Function::new(
            "Stdout".to_string(),
            "print".to_string(),
            Some(vec!()),
            Some(vec!(
                IO::new(&"String".to_string(), &"".to_string())
            )),
            Url::parse("file:///fake/file").unwrap(),
            "/flow0/stdout".to_string(),
            None,
            vec!(("".to_string(), 1, 0)),
            0);

        let br = Box::new(function) as Box<Runnable>;
        let code = runnable_to_code(&br);
        assert_eq!(code, "Function::new(\"print\", 0, false, vec!(), 0, STDOUT, None, vec!((\"\", 1, 0),))")
    }

    #[test]
    fn function_with_array_element_output() {
        let function = Function::new(
            "Stdout".to_string(),
            "print".to_string(),
            Some(vec!()),
            Some(vec!(
                IO::new(&"Array".to_string(), &"".to_string())
            )),
            Url::parse("file:///fake/file").unwrap(),
            "/flow0/stdout".to_string(),
            None,
            vec!(("0".to_string(), 1, 0)),
            0);

        let br = Box::new(function) as Box<Runnable>;
        let code = runnable_to_code(&br);
        assert_eq!(code, "Function::new(\"print\", 0, false, vec!(), 0, STDOUT, None, vec!((\"/0\", 1, 0),))")
    }
}
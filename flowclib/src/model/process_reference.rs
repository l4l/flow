use model::name::Name;
use model::name::HasName;
use model::route::Route;
use model::route::HasRoute;
use model::flow::Flow;
use model::function::Function;
use loader::loader::Validate;
use std::fmt;
use url::Url;

#[derive(Deserialize)]
pub struct ProcessReference {
    alias: Name,
    pub source: String,
    #[serde(skip_deserializing, default = "ProcessReference::default_url")]
    pub source_url: Url,
    #[serde(skip_deserializing)]
    pub process: Process
}

pub enum Process {
    FlowProcess(Flow),
    FunctionProcess(Function)
}

impl Default for Process {
    fn default() -> Process {
        Process::FlowProcess(Flow::default())
    }
}

impl HasName for ProcessReference {
    fn name(&self) -> &Name { &self.alias }
    fn alias(&self) -> &Name { &self.alias }
}

impl HasRoute for ProcessReference {
    fn route(&self) -> &Route {
        match self.process {
            Process::FlowProcess(ref flow) => {
                flow.route()
            },
            Process::FunctionProcess(ref function) => {
                function.route()
            }
        }
    }
}

impl Validate for ProcessReference {
    fn validate(&self) -> Result<(), String> {
        self.alias.validate()
    }
}

impl fmt::Display for ProcessReference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\t\t\t\talias: {}\n\t\t\t\t\tsource: {}\n\t\t\t\t\tURL: {}\n",
               self.alias, self.source, self.source_url)
    }
}

impl ProcessReference {
    fn default_url() -> Url {
        Url::parse("file::///").unwrap()
    }
}
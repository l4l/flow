use implementation::Implementation;
use std::collections::HashMap;
use provider::Provider;
use url::Url;

/*
    Implementations can be of two types - either a native and statically bound function referenced
    via a function reference, or WASM bytecode file that is interpreted at run-time that is
    referenced via a PathBuf pointing to the .wasm file
*/
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum ImplementationLocator<'a> {
    #[serde(skip_deserializing, skip_serializing)]
    Native(&'a dyn Implementation),
    Wasm(String),
}

/*
    Provided by libraries to help load and/or find implementations of processes
*/
#[derive(Deserialize, Serialize)]
pub struct ImplementationLocatorTable<'a> {
    pub locators: HashMap<String, ImplementationLocator<'a>>
}

impl<'a> ImplementationLocatorTable<'a> {
    pub fn new() -> Self {
        ImplementationLocatorTable {
            locators: HashMap::<String, ImplementationLocator<'a>>::new()
        }
    }

    pub fn load(provider: &Provider, url: &Url) -> Result<ImplementationLocatorTable<'a>, String> {
        let (resolved_url, _) = provider.resolve(url)?;
        let content = provider.get(&resolved_url)?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Could not read ILT from '{}'\nError = '{}'",
                                 url, e))
    }
}

#[cfg(test)]
mod test {
    extern crate url;

    use url::Url;

    use implementation_table::ImplementationLocatorTable;
    use implementation_table::ImplementationLocator;
    use implementation_table::ImplementationLocator::Wasm;
    use provider::Provider;

    pub struct TestProvider {
        test_content: &'static str
    }

    impl Provider for TestProvider {
        fn resolve(&self, url: &Url) -> Result<(Url, Option<String>), String> {
            Ok((url.clone(), None))
        }

        fn get(&self, _url: &Url) -> Result<String, String> {
            Ok(self.test_content.to_string())
        }
    }

    #[test]
    fn serialize() {
        let locator: ImplementationLocator = Wasm("add2.wasm".to_string());
        let mut ilt = ImplementationLocatorTable::new();
        ilt.locators.insert("//flowrlib/test-dyn-lib/add2".to_string(), locator);
        let serialized = serde_json::to_string_pretty(&ilt).unwrap();
        let expected = "{
  \"locators\": {
    \"//flowrlib/test-dyn-lib/add2\": \"add2.wasm\"
  }
}";
        assert_eq!(expected, serialized);
    }

    #[test]
    fn load_dyn_library() {
        let test_content = "{
  \"locators\": {
    \"//flowrlib/test-dyn-lib/add2\": \"add2.wasm\"
  }
}";
        let provider = TestProvider {
            test_content
        };
        let url = &Url::parse("file:://test/fake").unwrap();
        let ilt = ImplementationLocatorTable::load(&provider, url).unwrap();
        assert_eq!(ilt.locators.len(), 1);
        assert!(ilt.locators.get("//flowrlib/test-dyn-lib/add2").is_some());
        let locator = ilt.locators.get("//flowrlib/test-dyn-lib/add2").unwrap();
        match locator {
            Wasm(source) => assert_eq!(source, "add2.wasm"),
            _ => assert!(false, "Expected type 'Wasm' but found another type")
        }
    }
}



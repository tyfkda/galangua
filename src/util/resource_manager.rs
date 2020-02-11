use std::collections::HashMap;
use std::path::Path;

pub struct ResourceManager<T> {
    resource_map: HashMap<String, T>,
}

impl<T> ResourceManager<T> {
    pub fn new() -> ResourceManager<T> {
        ResourceManager {
            resource_map: HashMap::new(),
        }
    }

    pub fn load<F>(&mut self, base_path: &str, filenames: &Vec<&str>, loader: F) -> Result<(), String>
        where F : Fn(&str) -> Result<T, String>
    {
        for filename in filenames {
            let resource = loader(&format!("{}/{}", base_path, filename))?;
            let key = Path::new(filename).file_stem().unwrap().to_str().unwrap();
            self.resource_map.insert(String::from(key), resource);
        }

        Ok(())
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut T> {
        self.resource_map.get_mut(key)
    }
}

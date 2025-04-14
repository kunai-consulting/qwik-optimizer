#[derive(Default, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Config {
   pub transpile_jsx: bool,
}

impl Config {
    
    pub fn with_transpile_jsx(mut self, transpile_jsx: bool) -> Self {
        self.transpile_jsx = transpile_jsx;
        self
    }
}
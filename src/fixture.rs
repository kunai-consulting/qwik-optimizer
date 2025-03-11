use std::fs;
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Fixture{
    pub input: String,
    pub app: String,
    pub components: Vec<String>, 
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum FixtureSection{
    Input(String),
    App(String),
    Component(String),
}

impl FixtureSection {
    pub fn add_line(&mut self, line: String) {
        match self {
            FixtureSection::Input(ref mut s) => s.push_str(&line),
            FixtureSection::App(ref mut s) => s.push_str(&line),
            FixtureSection::Component(ref mut s) => s.push_str(&line),
        }
    }
    
}

impl Fixture {
    pub fn new(name: String) -> Self {
        let mut sections: Vec<FixtureSection> = Vec::new();
        let fix_path = PathBuf::from("./fixtures").join(format!("{}.fix", name));
        let fix_lines = fs::read_to_string(fix_path).unwrap();
        let fix_lines = fix_lines.lines(); 
        
        for line in fix_lines {
            match line  { 
               "## INPUT ##" => sections.push(FixtureSection::Input(String::new())),
                "## APP ##" => sections.push(FixtureSection::App(String::new())),
                "## COMPONENT ##" => sections.push(FixtureSection::Component(String::new())),
                line => 
                if let Some(last) = sections.last_mut() {
                    last.add_line(format!("{}\n", line));
                }
            }
        }
        
        let mut fix: Fixture = Default::default();
        
        for section in sections {
            match section {
                FixtureSection::Input(s) => fix.input = s,
                FixtureSection::App(s) => fix.app = s,
                FixtureSection::Component(s) => fix.components.push(s),
            }
        }
        
        fix
    }
}
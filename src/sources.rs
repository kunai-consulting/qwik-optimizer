use crate::prelude::*;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{parse_document, ParseOpts};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::fs;
use std::path::Path;

pub enum Container {
    Script(Vec<String>),
    HtmlDocument(RcDom),
}

impl Container {
    pub fn from_html_file(file_path: &Path) -> Result<Container> {
        let file_contents = fs::read(file_path)?;

        let opts = ParseOpts {
            tree_builder: TreeBuilderOpts {
                drop_doctype: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let dom = parse_document(RcDom::default(), opts)
            .from_utf8()
            .read_from(&mut file_contents.as_slice())?;

        Ok(Container::HtmlDocument(dom))
    }

    pub fn from_scripts(scripts: Vec<String>) -> Container {
        Container::Script(scripts)
    }

    pub fn from_script(script: &str) -> Container {
        let s = script.to_string();
        Container::Script(vec![s])
    }
    
}

pub trait ScriptSource {
    fn scripts(&self) -> Result<Vec<String>>;
    
    fn script_type() -> String;
}

fn extract_javascript(handle: &Handle) -> Vec<String> {
    let node = handle;
    let mut scripts: Vec<String> = Vec::new();

    extract_javascript_rec(node, &mut scripts);
    scripts
}

fn extract_javascript_rec(handle: &Handle, scripts: &mut Vec<String>) {
    let node = handle;

    match node.data {
        NodeData::Element { ref name, .. } if name.local.to_lowercase() == "script" => {
            node.children.borrow().iter().for_each(|child| {
                if let NodeData::Text { ref contents } = child.data {
                    let text = contents.borrow().to_string();
                    if !text.is_empty() {
                        scripts.push(text);
                    }
                }
            })
        }
        NodeData::ProcessingInstruction { .. } => unreachable!(),
        _ => {}
    }

    for child in node.children.borrow().iter() {
        extract_javascript_rec(child, scripts);
    }
}

impl ScriptSource for Container {
    fn scripts(&self) -> Result<Vec<String>> {
        match self {
            Container::Script(script) => Ok(script.clone()),
            Container::HtmlDocument(dom) => {
                let document = dom.document.clone();
                let scripts = extract_javascript(&document);
                Ok(scripts)
            }
        }
    }

    fn script_type() -> String {
        "js".to_string()
    }
}


use crate::yaml_fix::MyMap;
use dirgraphsvg::edges::EdgeType;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::ops::Deref;

pub mod check;
pub mod validation;

///
/// The main struct of this program
/// It describes a GSN element
///
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GsnNode {
    pub(crate) text: String,
    pub(crate) in_context_of: Option<Vec<String>>,
    pub(crate) supported_by: Option<Vec<String>>,
    pub(crate) classes: Option<Vec<String>>,
    pub(crate) url: Option<String>,
    pub(crate) level: Option<String>,
    #[serde(flatten)]
    pub(crate) additional: MyMap<String, String>,
    pub(crate) undeveloped: Option<bool>,
    #[serde(skip_deserializing)]
    pub module: String,
}

impl GsnNode {
    pub fn get_edges(&self) -> Vec<(String, EdgeType)> {
        let mut edges = Vec::new();
        if let Some(c_nodes) = &self.in_context_of {
            let mut es: Vec<(String, EdgeType)> = c_nodes
                .iter()
                .map(|target| (target.to_owned(), EdgeType::NoneToInContextOf))
                .collect();
            edges.append(&mut es);
        }
        if let Some(s_nodes) = &self.supported_by {
            let mut es: Vec<(String, EdgeType)> = s_nodes
                .iter()
                .map(|target| (target.to_owned(), EdgeType::NoneToSupportedBy))
                .collect();
            edges.append(&mut es);
        }
        edges
    }
}

///
/// Get root nodes
/// These are the unreferenced nodes.
///
fn get_root_nodes(nodes: &MyMap<String, GsnNode>) -> Vec<String> {
    let mut root_nodes: HashSet<String> = nodes.keys().cloned().collect();
    for node in nodes.values() {
        // Remove all keys if they are referenced; used to see if there is more than one top level node
        if let Some(context) = node.in_context_of.as_ref() {
            for cnode in context {
                root_nodes.remove(cnode);
            }
        }
        if let Some(support) = node.supported_by.as_ref() {
            for snode in support {
                root_nodes.remove(snode);
            }
        }
    }
    Vec::from_iter(root_nodes)
}

///
/// Gathers all different 'level' attributes from all nodes.
///
pub fn get_levels(nodes: &MyMap<String, GsnNode>) -> BTreeMap<&str, Vec<&str>> {
    let mut levels = BTreeMap::<&str, Vec<&str>>::new();
    for (id, node) in nodes.iter() {
        if let Some(l) = &node.level {
            levels.entry(l.trim()).or_insert(Vec::new()).push(id);
        }
    }
    levels
}

#[derive(Debug, PartialEq, Serialize)]
pub enum ModuleDependency {
    SupportedBy,
    InContextOf,
    Composite,
}

///
/// Calculate module dependencies
///
///
pub fn calculate_module_dependencies(
    nodes: &MyMap<String, GsnNode>,
) -> BTreeMap<String, BTreeMap<String, ModuleDependency>> {
    let mut res = BTreeMap::<String, BTreeMap<String, ModuleDependency>>::new();
    for v in nodes.values() {
        res.insert(v.module.to_owned(), BTreeMap::new());
    }
    for v in nodes.deref().values() {
        if let Some(sups) = &v.supported_by {
            for sup in sups {
                let other_module = &nodes.get(sup).unwrap().module;
                if &v.module != other_module {
                    let e = res.get_mut(&v.module).unwrap();
                    e.entry(other_module.to_owned())
                        .and_modify(|x| {
                            if *x == ModuleDependency::InContextOf {
                                *x = ModuleDependency::Composite
                            }
                        })
                        .or_insert(ModuleDependency::SupportedBy);
                }
            }
        }
        if let Some(ctxs) = &v.in_context_of {
            for ctx in ctxs {
                let other_module = &nodes.get(ctx).unwrap().module;
                if &v.module != other_module {
                    let e = res.get_mut(&v.module).unwrap();
                    e.entry(other_module.to_owned())
                        .and_modify(|x| {
                            if *x == ModuleDependency::SupportedBy {
                                *x = ModuleDependency::Composite
                            }
                        })
                        .or_insert(ModuleDependency::InContextOf);
                }
            }
        }
    }
    res
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_level_exists() {
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert("Sn1".to_owned(), Default::default());
        let output = get_levels(&nodes);
        assert!(output.is_empty());
    }

    #[test]
    fn two_levels_exist() {
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "Sn1".to_owned(),
            GsnNode {
                level: Some("x1".to_owned()),
                ..Default::default()
            },
        );
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                level: Some("x2".to_owned()),
                ..Default::default()
            },
        );
        let output = get_levels(&nodes);
        assert_eq!(output.len(), 2);
        assert!(output.contains_key(&"x1"));
        assert!(output.contains_key(&"x2"));
    }
}

use crate::dirgraphsvg::edges::EdgeType;
use crate::dirgraphsvg::{escape_node_id, escape_text, nodes::*};
use crate::gsn::{get_levels, GsnNode, Module};
use anyhow::{anyhow, Result};
use chrono::Utc;
use clap::ArgMatches;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::{Component, PathBuf};
use std::rc::Rc;

#[derive(Default, Eq, PartialEq)]
pub enum RenderLegend {
    No,
    #[default]
    Short,
    Full,
}

#[derive(Default)]
pub struct RenderOptions {
    pub stylesheets: Vec<String>,
    pub layers: Vec<String>,
    pub legend: RenderLegend,
    pub embed_stylesheets: bool,
    pub architecture_filename: Option<String>,
    pub evidences_filename: Option<String>,
    pub complete_filename: Option<String>,
    pub skip_argument: bool,
    pub skip_architecture: bool,
    pub skip_evidences: bool,
    pub skip_complete: bool,
}

impl From<&ArgMatches> for RenderOptions {
    fn from(matches: &ArgMatches) -> Self {
        let legend = if matches.get_flag("NO_LEGEND") {
            RenderLegend::No
        } else if matches.get_flag("FULL_LEGEND") {
            RenderLegend::Full
        } else {
            RenderLegend::Short
        };
        let layers = matches
            .get_many::<String>("LAYERS")
            .into_iter()
            .flatten()
            .cloned()
            .collect::<Vec<_>>();
        let architecture_filename = matches.get_one::<String>("ARCHITECTURE_VIEW").cloned();
        let complete_filename = matches.get_one::<String>("COMPLETE_VIEW").cloned();
        let evidences_filename = matches.get_one::<String>("EVIDENCES").cloned();
        RenderOptions {
            stylesheets: matches
                .get_many::<String>("STYLESHEETS")
                .into_iter()
                .flatten()
                .cloned()
                .collect::<Vec<_>>(),
            layers,
            legend,
            embed_stylesheets: matches.get_flag("EMBED_CSS"),
            architecture_filename,
            evidences_filename,
            complete_filename,
            skip_argument: matches.get_flag("NO_ARGUMENT_VIEW"),
            skip_architecture: matches.get_flag("NO_ARCHITECTURE_VIEW"),
            skip_evidences: matches.get_flag("NO_EVIDENCES"),
            skip_complete: matches.get_flag("NO_COMPLETE_VIEW"),
        }
    }
}

///
///
///
///
///
pub fn svg_from_gsn_node(
    id: &str,
    gsn_node: &GsnNode,
    layers: &[String],
) -> Rc<RefCell<dyn crate::dirgraphsvg::nodes::Node>> {
    let classes = node_classes_from_node(gsn_node);
    // Add layer to node output
    let node_text = node_text_from_node_and_layers(gsn_node, layers);
    // Create node
    match id {
        id if id.starts_with('G') => new_goal(
            id,
            &node_text,
            gsn_node.undeveloped.unwrap_or(false),
            gsn_node.url.to_owned(),
            classes,
        ),
        id if id.starts_with("Sn") => {
            new_solution(id, &node_text, gsn_node.url.to_owned(), classes)
        }
        id if id.starts_with('S') => new_strategy(
            id,
            &node_text,
            gsn_node.undeveloped.unwrap_or(false),
            gsn_node.url.to_owned(),
            classes,
        ),
        id if id.starts_with('C') => new_context(id, &node_text, gsn_node.url.to_owned(), classes),
        id if id.starts_with('A') => {
            new_assumption(id, &node_text, gsn_node.url.to_owned(), classes)
        }
        id if id.starts_with('J') => {
            new_justification(id, &node_text, gsn_node.url.to_owned(), classes)
        }
        _ => unreachable!(),
    }
}

///
/// Create SVG node text from GsnNode and layer information
///
///
fn node_text_from_node_and_layers(gsn_node: &GsnNode, layers: &[String]) -> String {
    let mut node_text = gsn_node.text.to_owned();
    let mut additional_text = vec![];
    for layer in layers {
        if let Some(layer_text) = gsn_node.additional.get(layer) {
            additional_text.push(format!(
                "\n{}: {}",
                layer.to_ascii_uppercase(),
                layer_text.replace('\n', " ")
            ));
        }
    }
    if !additional_text.is_empty() {
        node_text.push_str("\n\n");
        node_text.push_str(&additional_text.join("\n"));
    }
    node_text
}

///
///
///
fn node_classes_from_node(gsn_node: &GsnNode) -> Vec<String> {
    let layer_classes: Option<Vec<String>> = gsn_node
        .additional
        .keys()
        .map(|k| {
            let mut t = escape_text(&k.to_ascii_lowercase());
            t.insert_str(0, "gsn_");
            Some(t.to_owned())
        })
        .collect();
    let mut mod_class = gsn_node.module.to_owned();
    mod_class.insert_str(0, "gsn_module_");
    let classes = gsn_node
        .classes
        .iter()
        .chain(layer_classes.iter())
        .flatten()
        .chain(&[mod_class])
        .cloned()
        .collect();
    classes
}

///
///
///
///
///
pub fn away_svg_from_gsn_node(
    id: &str,
    gsn_node: &GsnNode,
    module: &Module,
    source_module: &Module,
    layers: &[String],
) -> Result<Rc<RefCell<dyn crate::dirgraphsvg::nodes::Node>>> {
    let classes = node_classes_from_node(gsn_node);

    let mut module_url = get_relative_module_url(&module.filename, &source_module.filename)?;
    module_url.push('#');
    module_url.push_str(&escape_node_id(id));

    // Add layer to node output
    let node_text = node_text_from_node_and_layers(gsn_node, layers);

    // Create node
    Ok(match id {
        id if id.starts_with('G') => new_away_goal(
            id,
            &node_text,
            &gsn_node.module,
            Some(module_url),
            gsn_node.url.to_owned(),
            classes,
        ),
        id if id.starts_with("Sn") => new_away_solution(
            id,
            &node_text,
            &gsn_node.module,
            Some(module_url),
            gsn_node.url.to_owned(),
            classes,
        ),
        id if id.starts_with('S') => new_strategy(
            id,
            &node_text,
            gsn_node.undeveloped.unwrap_or(false),
            gsn_node.url.to_owned(),
            classes,
        ),
        id if id.starts_with('C') => new_away_context(
            id,
            &node_text,
            &gsn_node.module,
            Some(module_url),
            gsn_node.url.to_owned(),
            classes,
        ),
        id if id.starts_with('A') => new_away_assumption(
            id,
            &node_text,
            &gsn_node.module,
            Some(module_url),
            gsn_node.url.to_owned(),
            classes,
        ),
        id if id.starts_with('J') => new_away_justification(
            id,
            &node_text,
            &gsn_node.module,
            Some(module_url),
            gsn_node.url.to_owned(),
            classes,
        ),
        _ => unreachable!(), // Prefixes are checked during validation.
    })
}

///
///
///
fn get_relative_module_url(target: &str, source: &str) -> Result<String> {
    let source_canon = std::path::PathBuf::from(&source).canonicalize()?;
    let source_canon = source_canon
        .parent()
        .ok_or_else(|| anyhow!("No parent for {}", source))?;
    let target_canon = std::path::PathBuf::from(&target).canonicalize()?;
    let source_comps: Vec<Component> = source_canon.components().collect();
    let mut source_comps_iter = source_comps.iter();
    let target_comps = target_canon.components();
    let mut diff_comps: PathBuf = target_comps
        .skip_while(|t| source_comps_iter.next().map(|x| x == t).unwrap_or(false))
        .collect();
    let mut prefix = match source_comps_iter.count() {
        x if x == 0 => "./".to_owned(),
        x if x > 0 => "../".repeat(x + 1),
        _ => unreachable!(),
    };
    diff_comps.set_extension("svg");
    prefix.push_str(&diff_comps.to_string_lossy());
    Ok(prefix)
}

///
///
///
pub fn render_architecture(
    output: &mut impl Write,
    modules: &HashMap<String, Module>,
    dependencies: BTreeMap<String, BTreeMap<String, EdgeType>>,
    render_options: &RenderOptions,
) -> Result<()> {
    let mut dg = crate::dirgraphsvg::DirGraph::default();
    let svg_nodes: BTreeMap<String, Rc<RefCell<dyn Node>>> = modules
        .iter()
        .filter(|(k, _)| dependencies.contains_key(k.to_owned()))
        .map(|(k, module)| {
            (
                k.to_owned(),
                new_module(
                    k,
                    module
                        .meta
                        .as_ref()
                        .and_then(|m| m.brief.to_owned())
                        .unwrap_or_else(|| "".to_owned())
                        .as_str(),
                    get_relative_module_url(&module.filename, &module.filename).ok(),
                    vec![],
                ) as Rc<RefCell<dyn Node>>,
            )
        })
        .collect();
    let mut edges: BTreeMap<String, Vec<(String, EdgeType)>> = dependencies
        .into_iter()
        .map(|(k, v)| (k, Vec::from_iter(v.into_iter())))
        .collect();

    dg = dg
        .add_nodes(svg_nodes)
        .add_edges(&mut edges)
        .embed_stylesheets(render_options.embed_stylesheets)
        .add_css_stylesheets(
            &mut render_options
                .stylesheets
                .iter()
                .map(AsRef::as_ref)
                .collect(),
        );

    dg.write(output, true)?;

    Ok(())
}

///
/// Render all nodes in one diagram
///
/// TODO mask modules MASK_MODULE
///
pub fn render_complete(
    output: &mut impl Write,
    nodes: &BTreeMap<String, GsnNode>,
    render_options: &RenderOptions,
) -> Result<()> {
    // let masked_modules_opt = matches
    //     .values_of("MASK_MODULE")
    //     .map(|x| x.map(|y| y.to_owned()).collect::<Vec<String>>());
    // let masked_modules = masked_modules_opt.iter().flatten().collect::<Vec<_>>();
    let mut dg = crate::dirgraphsvg::DirGraph::default();
    let mut edges: BTreeMap<String, Vec<(String, EdgeType)>> = nodes
        .iter()
        // .filter(|(_, node)| !masked_modules.contains(&&node.module))
        // TODO continue masking here
        .map(|(id, node)| (id.to_owned(), node.get_edges()))
        .collect();
    let svg_nodes: BTreeMap<String, Rc<RefCell<dyn Node>>> = nodes
        .iter()
        .map(|(id, node)| {
            (
                id.to_owned(),
                svg_from_gsn_node(id, node, &render_options.layers),
            )
        })
        .collect();
    dg = dg
        .add_nodes(svg_nodes)
        .add_edges(&mut edges)
        .add_levels(&get_levels(nodes))
        .embed_stylesheets(render_options.embed_stylesheets)
        .add_css_stylesheets(
            &mut render_options
                .stylesheets
                .iter()
                .map(AsRef::as_ref)
                .collect(),
        );

    dg.write(output, false)?;

    Ok(())
}

///
/// Render all nodes in one diagram
///
///  1) Map gsn nodes to svg nodes
///     foreign module nodes will be mapped to the away svg node
///  2) Replace the edges with the right ones
///  3) filter all foreign modules that have no edge to this module
///
pub fn render_argument(
    output: &mut impl Write,
    module_name: &str,
    modules: &HashMap<String, Module>,
    nodes: &BTreeMap<String, GsnNode>,
    render_options: &RenderOptions,
) -> Result<()> {
    let mut dg = crate::dirgraphsvg::DirGraph::default();
    let mut svg_nodes: BTreeMap<String, Rc<RefCell<dyn Node>>> = nodes
        .iter()
        .filter(|(_, node)| node.module == module_name)
        .map(|(id, node)| {
            (
                id.to_owned(),
                svg_from_gsn_node(id, node, &render_options.layers),
            )
        })
        .collect();

    svg_nodes.append(
        &mut nodes
            .iter()
            .filter(|(_, node)| node.module != module_name)
            .map(|(id, node)| {
                Ok((
                    id.to_owned(),
                    away_svg_from_gsn_node(
                        id,
                        node,
                        // unwraps are ok, since node.module and modules are consistently created
                        modules.get(&node.module).unwrap(),
                        modules.get(module_name).unwrap(),
                        &render_options.layers,
                    )?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?,
    );

    let mut edges: BTreeMap<String, Vec<(String, EdgeType)>> = nodes
        .iter()
        .map(|(id, node)| {
            (
                id.to_owned(),
                node.get_edges()
                    .into_iter()
                    .filter(|(target, _)| {
                        !(node.module != module_name
                            // unwrap is ok, since all references are checked at the beginning
                            && nodes.get(target).unwrap().module != module_name)
                    })
                    .collect::<Vec<(String, EdgeType)>>(),
            )
        })
        .filter(|(_, targets)| !targets.is_empty())
        .collect();

    svg_nodes.retain(|id, _| {
        edges.contains_key(id)
            || edges.values().flatten().any(|(x, _)| x == id)
            // unwrap is ok, since all references are checked at the beginning
            || nodes.get(id).unwrap().module == module_name
    });

    dg = dg
        .add_nodes(svg_nodes)
        .add_edges(&mut edges)
        .add_levels(&get_levels(nodes))
        .embed_stylesheets(render_options.embed_stylesheets)
        .add_css_stylesheets(
            &mut render_options
                .stylesheets
                .iter()
                .map(AsRef::as_ref)
                .collect(),
        );

    // Add meta information if requested
    if render_options.legend != RenderLegend::No {
        let mut meta_info = vec![format!("Generated on: {}", Utc::now())];
        if let Some(Some(meta)) = &modules.get(module_name).map(|x| &x.meta) {
            meta_info.insert(0, format!("Module: {}", meta.name));
            if let Some(brief) = &meta.brief {
                meta_info.insert(1, brief.to_owned());
            }
            if render_options.legend == RenderLegend::Full {
                let add = format!("{:?}", meta.additional);
                meta_info.append(&mut add.lines().map(|x| x.to_owned()).collect::<Vec<String>>());
            }
        }
        dg = dg.add_meta_information(&mut meta_info);
    }

    dg.write(output, false)?;

    Ok(())
}

///
/// Output list of evidences.
///
/// No template engine is used in order to keep dependencies to a minimum.
///
///
pub(crate) fn render_evidences(
    output: &mut impl Write,
    nodes: &BTreeMap<String, GsnNode>,
    render_options: &RenderOptions,
) -> Result<()> {
    writeln!(output)?;
    writeln!(output, "List of Evidences")?;
    writeln!(output)?;

    let solutions: Vec<(&String, &GsnNode)> = nodes
        .iter()
        .filter(|(id, _)| id.starts_with("Sn"))
        .collect();
    if solutions.is_empty() {
        writeln!(output, "No evidences found.")?;
    }
    let width = (solutions.len() as f32).log10().ceil() as usize;
    for (i, (id, node)) in solutions.into_iter().enumerate() {
        writeln!(
            output,
            "{:>width$}. {}: {}",
            i + 1,
            id,
            node.text
                .replace('\n', &format!("\n{: >w$}", ' ', w = width + 4 + id.len()))
        )?;
        let width = width + 2;
        writeln!(output)?;
        writeln!(output, "{: >width$}{}", ' ', node.module)?;
        writeln!(output)?;
        if let Some(url) = &node.url {
            writeln!(output, "{: >width$}{}", ' ', url)?;
            writeln!(output)?;
        }
        for (layer, text) in node
            .additional
            .iter()
            .filter(|(l, _)| render_options.layers.iter().any(|x| &x == l))
        {
            writeln!(
                output,
                "{: >width$}{}: {}",
                ' ',
                layer.to_ascii_uppercase(),
                text.replace(
                    '\n',
                    &format!("\n{: >w$}", ' ', w = width + 2 + layer.len())
                )
            )?;
            writeln!(output)?;
        }
    }

    Ok(())
}

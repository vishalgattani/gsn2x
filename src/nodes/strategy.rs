use super::box_node::BoxNode;

#[derive(Clone)]
pub struct Strategy;

impl Strategy {
    pub fn new(
        id: &str,
        text: &str,
        undeveloped: bool,
        url: Option<String>,
        classes: Option<Vec<String>>,
        forced_level: Option<u32>,
    ) -> BoxNode {
        BoxNode::new(id, text, undeveloped, 15, url, classes, forced_level)
    }
}

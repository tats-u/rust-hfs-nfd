use ahash::AHashMap;

/// Tree node struct for reverse (HFS+ to normal) dictionary
pub struct ReverseTreeNode {
    /// A character composed of the subcomponents used for the search
    pub current: Option<char>,
    /// Subdictionary for trailing characters
    pub next: Option<Box<AHashMap<char, ReverseTreeNode>>>,
}

impl ReverseTreeNode {
    /// Create a node instance.
    pub fn new(current: Option<char>, next: Option<Box<AHashMap<char, ReverseTreeNode>>>) -> Self {
        return Self {
            current: current,
            next: next,
        };
    }
}

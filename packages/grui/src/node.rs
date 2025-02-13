pub struct Node {
    children: Vec<Node>,
}

impl Node {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

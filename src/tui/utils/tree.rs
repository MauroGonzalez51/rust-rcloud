pub type TreeNodeRef<T> = std::rc::Rc<std::cell::RefCell<TreeNode<T>>>;

#[derive(Clone, Debug)]
pub struct TreeNode<T: Clone> {
    pub value: T,
    children: Vec<TreeNodeRef<T>>,
    parent: Option<std::rc::Weak<std::cell::RefCell<TreeNode<T>>>>,
}

impl<T: Clone> TreeNode<T> {
    pub fn new(value: T) -> TreeNodeRef<T> {
        std::rc::Rc::new(std::cell::RefCell::new(Self {
            value,
            children: Vec::new(),
            parent: None,
        }))
    }

    pub fn children(&self) -> &Vec<TreeNodeRef<T>> {
        &self.children
    }

    pub fn parent(&self) -> Option<TreeNodeRef<T>> {
        self.parent.as_ref()?.upgrade()
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }
}

pub enum TreeNodeGetBy<T> {
    Index(usize),
    Value(T),
}

pub trait TreeNodeOperations<T: Clone + PartialEq> {
    fn push(&self, child: TreeNodeRef<T>);
    fn get(&self, node: TreeNodeGetBy<T>) -> Option<TreeNodeRef<T>>;
    fn find_recursive(&self, value: &T) -> Option<TreeNodeRef<T>>;
}

impl<T: Clone + PartialEq> TreeNodeOperations<T> for TreeNodeRef<T> {
    fn push(&self, child: TreeNodeRef<T>) {
        self.borrow_mut().children.push(child.clone());
        child.borrow_mut().parent = Some(std::rc::Rc::downgrade(self));
    }

    fn get(&self, node: TreeNodeGetBy<T>) -> Option<TreeNodeRef<T>> {
        match node {
            TreeNodeGetBy::Index(index) => self.borrow().children.get(index).cloned(),
            TreeNodeGetBy::Value(value) => self.find_recursive(&value),
        }
    }

    fn find_recursive(&self, value: &T) -> Option<TreeNodeRef<T>> {
        if self.borrow().value == *value {
            return Some(self.clone());
        }

        for child in self.borrow().children.iter() {
            if let Some(found) = child.find_recursive(value) {
                return Some(found);
            }
        }

        None
    }
}

#[derive(Clone, Debug)]
pub struct TreeBuilder<T: Clone> {
    value: T,
    children: Vec<TreeBuilder<T>>,
}

impl<T: Clone + PartialEq> TreeBuilder<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            children: Vec::new(),
        }
    }

    pub fn with_children(mut self, children: Vec<TreeBuilder<T>>) -> Self {
        self.children = children;
        self
    }

    pub fn child(mut self, child: TreeBuilder<T>) -> Self {
        self.children.push(child);
        self
    }

    pub fn build(self) -> TreeNodeRef<T> {
        let node = TreeNode::new(self.value);

        for child in self.children {
            node.push(child.build())
        }

        node
    }
}

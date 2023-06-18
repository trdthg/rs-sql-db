use std::cell::RefCell;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::rc::{Rc, Weak};

use super::insert::*;

use serde_derive::{Deserialize, Serialize};

pub const MAX_DEGREE: usize = 5;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LinkType {
    Branch(Rc<RefCell<BranchNode>>),
    Leaf(Rc<RefCell<LeafNode>>),
    Data(DataNode),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: Option<usize>,
    pub link: LinkType,
}
impl Node {
    pub fn new(id: usize, link: LinkType) -> Node {
        Node { id: Some(id), link }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BranchNode {
    pub ids: Vec<Node>,
    pub father: Option<Weak<RefCell<BranchNode>>>,
}
impl BranchNode {
    pub fn new(ids: Vec<Node>) -> BranchNode {
        BranchNode { ids, father: None }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DataNode {
    pub id: usize,
    pub data: String,
}
impl DataNode {
    pub fn new(id: usize, data: &str) -> Self {
        DataNode {
            id,
            data: data.to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeafNode {
    pub ids: Vec<DataNode>,
    pub father: Option<Weak<RefCell<BranchNode>>>,
    pub next: Option<Rc<RefCell<LeafNode>>>,
}
impl LeafNode {
    pub fn new(id: usize, data: &str) -> LeafNode {
        let ids = vec![DataNode::new(id, data)];
        LeafNode {
            ids,
            next: None,
            father: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BPlusTree {
    pub name: String,
    pub root: Option<LinkType>,
}

impl BPlusTree {
    pub fn new(name: &str) -> BPlusTree {
        BPlusTree {
            name: name.to_string(),
            root: None,
        }
    }

    pub fn de(name: &str) -> Self {
        let f = File::open(name).unwrap();
        let mut s = String::new();
        let mut reader = BufReader::new(f);
        reader.read_to_string(&mut s).unwrap();
        serde_json::from_str::<Self>(&s).unwrap()
    }

    pub fn se(&self, name: &str) {
        let sered = serde_json::to_string(self).unwrap();
        let mut f = File::create(name).unwrap();
        f.write(sered.as_bytes()).unwrap();
    }

    pub fn insert(&mut self, id: usize, data: &str) {
        match &self.root.clone() {
            Some(LinkType::Leaf(node)) => {
                let len = insert_leaf(node, id, data);
                if len == MAX_DEGREE {
                    self.root = Some(LinkType::Branch(splite_leaf(&node.clone())));
                }
            }
            Some(LinkType::Branch(_node)) => {
                if let Some(new_root) = find_leaf(_node, id, data) {
                    self.root = Some(LinkType::Branch(new_root));
                }
            }
            None => {
                let new_branch = LeafNode::new(id, data);
                self.root = Some(LinkType::Leaf(Rc::new(RefCell::new(new_branch))));
            }
            _ => {}
        }
    }

    pub fn select(&self, id: usize) -> Option<DataNode> {
        match &self.root.clone() {
            Some(LinkType::Leaf(leaf)) => {
                let mut res = None;
                for tuple in &leaf.borrow().ids {
                    if id == tuple.id {
                        res = Some(tuple.clone())
                    }
                }
                return res;
            }
            Some(LinkType::Branch(_node)) => None,
            _ => None,
        }
    }

    pub fn travel(&self) {
        // TODO
        let mut vec = Vec::new();
        let mut vec2 = Vec::new();
        match self.root.clone() {
            Some(LinkType::Branch(node)) => {
                vec.push(node.clone());
                while vec.len() != 0 {
                    let a = vec.pop().unwrap();
                    for tuple in a.borrow().ids.clone() {
                        println!("{:?}", tuple.id);
                        match tuple.link {
                            LinkType::Branch(branch) => {
                                vec.insert(0, branch.clone());
                            }
                            LinkType::Leaf(leaf) => {
                                vec2.insert(0, leaf);
                            }
                            _ => {}
                        }
                    }
                }
                while vec2.len() != 0 {
                    let leaf = vec2.pop();
                    for tuple in &leaf.unwrap().borrow().ids {
                        // println!("{:?}", tuple.id);
                    }
                }
            }
            Some(LinkType::Leaf(leaf)) => {
                for tuple in leaf.borrow().ids.clone() {
                    println!("{:?}", tuple.id);
                }
            }
            _ => {}
        }
    }

    pub fn se_self(&mut self) {
        let mut vec = Vec::new();
        let mut vec2 = Vec::new();
        match self.root.clone() {
            Some(LinkType::Branch(node)) => {
                vec.push(node.clone());
                while vec.len() != 0 {
                    let a = vec.pop().unwrap();
                    for tuple in a.borrow().ids.clone() {
                        println!("{:?}", tuple.id);
                        match tuple.link {
                            LinkType::Branch(branch) => {
                                vec.insert(0, branch.clone());
                            }
                            LinkType::Leaf(leaf) => {
                                vec2.insert(0, leaf);
                            }
                            _ => {}
                        }
                    }
                }
                while vec2.len() != 0 {
                    let leaf = vec2.pop();
                    for tuple in &leaf.unwrap().borrow().ids {
                        // println!("{:?}", tuple.id);
                    }
                }
            }
            Some(LinkType::Leaf(leaf)) => {
                for tuple in leaf.borrow().ids.clone() {
                    println!("{:?}", tuple.id);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod test {
    use std::mem::size_of;

    use super::*;
    #[test]
    fn a() {
        let now = std::time::Instant::now();
        let mut tree = BPlusTree::new("tmp");
        for i in 1..=19 {
            println!("------------------{}----------------", i);
            tree.insert(i, "sss");
        }
        // tree.insert(6, DataNode { id: 6, name: "".to_string() });
        // tree.insert(1, DataNode {});
        // tree.insert(2, DataNode {});
        // tree.insert(3, DataNode {});
        // tree.insert(4, DataNode {});
        // tree.insert(5, DataNode {});
        // tree.insert(6, DataNode {});
        // tree.insert(7, DataNode {});
        // tree.insert(8, DataNode {});
        // tree.insert(9, DataNode {});
        // tree.insert(10, DataNode {});
        // tree.insert(11, DataNode {});
        // tree.insert(12, DataNode {});
        // tree.insert(13, DataNode {});
        // tree.insert(14, DataNode {});
        // tree.insert(15, DataNode {});
        // tree.insert(16, DataNode {});
        // tree.insert(17, DataNode {});
        // tree.insert(18, DataNode {});
        // tree.insert(19, DataNode {});
        println!("------------------------------------");
        tree.travel();
        println!("{:?}", now.elapsed());

        // println!("{:?}", tree);
        // println!("{:#?}", tree);
    }

    #[test]
    fn a2() {
        use std::mem::size_of;
        println!("bplustree: {}", size_of::<BPlusTree>());
        println!("linktype: {}", size_of::<LinkType>());
        println!("branchnode: {}", size_of::<BranchNode>());
        println!("leafnode: {}", size_of::<LeafNode>());
        println!("datanode: {}", size_of::<DataNode>());
        println!("father: {}", size_of::<Option<Weak<RefCell<BranchNode>>>>());
    }

    #[test]
    fn b() {
        let mut s = vec![0, 1, 1, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55];
        let num = 42;
        // match s.binary_search(&num) {
        //     Ok(pos) => {} // element already in vector @ `pos`
        //     Err(pos) => s.insert(pos, num),
        // }
        let mut s = vec![1, 5, 9];
        let num = 9;
        let idx = s.binary_search(&num).unwrap_or_else(|x| x);
        match s.get(idx) {
            Some(_) => {
                // 若有就插入
            }
            None => {
                // 若为空就修改
            }
        }
        s.insert(idx, num);
        println!("{:#?} {:?}", idx, s);
    }

    #[test]
    fn c() {
        let a = vec![1, 1, 1, 1, 1];
        let b = a[0..1].to_owned();
        println!("{:?}", &a[1..4]);
    }

    #[test]
    fn d() {
        let a = Rc::new(RefCell::new(1));
        *a.borrow_mut() = 2;
        *a.borrow_mut() = 3;
        a.borrow();
        let c = a.borrow_mut();
        println!("{:?}", *c);
    }
}

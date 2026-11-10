pub type NodeId = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeType {
    Honest,
    Target,
    Sybil,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub id: NodeId,
    pub node_type: NodeType,
    pub inbound_peers: Vec<NodeId>,
    pub outbound_peers: Vec<NodeId>,
    pub max_peers: usize,
}

impl Node {
    pub fn new(id: NodeId, node_type: NodeType, max_peers: usize) -> Self {
        Self {
            id,
            node_type,
            inbound_peers: Vec::new(),
            outbound_peers: Vec::new(),
            max_peers,
        }
    }
}

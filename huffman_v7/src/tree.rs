#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    OneByte(u8),
    TwoBytes(u16),
}

#[derive(Clone, Debug, Eq)]
pub struct Node {
    pub frequency: u32,
    pub symbol: Symbol,
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
}

impl Node {
    pub fn new(symbol: Symbol, frequency: u32) -> Self {
        Self {
            frequency,
            symbol,
            left: None,
            right: None,
        }
    }

    pub fn new_parent(left: Node, right: Node) -> Self {
        Self {
            frequency: left.frequency + right.frequency,
            symbol: left.symbol,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.frequency == other.frequency
    }
}
impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match other.frequency.cmp(&self.frequency) {
            std::cmp::Ordering::Equal => match (self.symbol, other.symbol) {
                (Symbol::OneByte(a), Symbol::OneByte(b)) => b.cmp(&a),
                (Symbol::TwoBytes(a), Symbol::TwoBytes(b)) => b.cmp(&a),
                (Symbol::OneByte(_), Symbol::TwoBytes(_)) => std::cmp::Ordering::Greater,
                (Symbol::TwoBytes(_), Symbol::OneByte(_)) => std::cmp::Ordering::Less,
            },
            other_ordering => other_ordering,
        }
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

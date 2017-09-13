#[derive(Debug)]
pub enum G930Keys {
    G1,
    G2,
    G3,
    UNKNOWN  // Apparently, KeyUp events are not associated with any key.
}

#[derive(Debug, PartialEq, Eq)]
pub enum G930KeyEventTypes {
    KeyDown,
    KeyUp
}

#[derive(Debug)]
pub struct G930KeyEvent {
    pub key: G930Keys,
    pub type_: G930KeyEventTypes
}

impl G930KeyEvent {
    pub fn new(key: G930Keys, type_: G930KeyEventTypes) -> Self {
        Self {
            key,
            type_
        }
    }
}

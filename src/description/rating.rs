use std::cmp::{Ord, PartialEq, Eq, PartialOrd};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Ord, PartialOrd, Eq)]
#[serde(tag = "type",rename_all = "camelCase")]
pub enum Rating {
    Low,
    Medium,
    High
}

#[test]
fn ratings_test() {
    assert!( Rating::Low < Rating::Medium );
    assert!( Rating::Low < Rating::High );
    assert!( Rating::Medium < Rating::High );
    assert!( Rating::High > Rating::Low );
    assert!( Rating::High > Rating::Medium );
    assert!( Rating::Medium > Rating::Low );
    assert!( Rating:: Low == Rating::Low );
    assert!( Rating:: Medium == Rating::Medium );
    assert!( Rating:: High == Rating::High );
}
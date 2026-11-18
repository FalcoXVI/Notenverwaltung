use super::{Category, Entry, Subject};

/// A category together with all of its entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryData {
    pub category: Category,
    pub entries: Vec<Entry>,
}

/// A subject with everything needed to calculate its grade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectData {
    pub subject: Subject,
    pub categories: Vec<CategoryData>,
}

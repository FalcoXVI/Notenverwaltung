use crate::domain::{CategoryId, CategoryKind, GradingKey, SubjectId};

/// A category as edited in the subject dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryDraft {
    /// `None` for a category that does not exist in the database yet.
    pub id: Option<CategoryId>,
    pub name: String,
    pub weight_percent: u32,
    pub kind: CategoryKind,
    pub best_n: Option<u32>,
}

/// A subject with all its categories as edited in the subject dialog.
/// Saving it replaces the stored categories by exactly this list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectDraft {
    /// `None` for a new subject.
    pub id: Option<SubjectId>,
    pub name: String,
    pub key: GradingKey,
    pub categories: Vec<CategoryDraft>,
}

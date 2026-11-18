//! Typed database ids. Each table gets its own id type so the compiler
//! rejects e.g. passing a subject id where a category id is expected.

/// Generates a newtype wrapper around `i64` for one kind of id.
macro_rules! define_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(pub i64);
    };
}

define_id!(
    /// Primary key of a semester.
    SemesterId
);
define_id!(
    /// Primary key of a subject.
    SubjectId
);
define_id!(
    /// Primary key of a category.
    CategoryId
);
define_id!(
    /// Primary key of an entry.
    EntryId
);

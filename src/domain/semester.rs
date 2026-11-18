use super::SemesterId;

/// One semester. Each semester stands on its own; there is no year grade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Semester {
    pub id: SemesterId,
    /// Running number (1st, 2nd, … semester).
    pub number: u32,
    /// Optional free label such as "WS 2026/27"; may be empty.
    pub label: String,
}

impl Semester {
    /// Name for lists and headings, e.g. "3. Semester (WS 2026/27)".
    pub fn display_name(&self) -> String {
        if self.label.is_empty() {
            format!("{}. Semester", self.number)
        } else {
            format!("{}. Semester ({})", self.number, self.label)
        }
    }
}

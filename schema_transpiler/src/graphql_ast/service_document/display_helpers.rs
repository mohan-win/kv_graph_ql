use std::fmt;

/// Writes graphQL description, followed by a new-line(\n).
pub fn write_description_ln(f: &mut fmt::Formatter, desc: &Option<String>) -> fmt::Result {
    if desc.is_some() {
        write!(f, "\"\"\"{}\"\"\"\n", desc.as_ref().unwrap())
    } else {
        Ok(())
    }
}

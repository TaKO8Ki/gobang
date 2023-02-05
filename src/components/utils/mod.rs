pub mod input;
pub mod scroll_vertical;

pub fn is_whitespace(c: char) -> bool {
    c.is_whitespace()
}

pub fn is_nonalphanumeric(c: char) -> bool {
    !c.is_alphanumeric()
}

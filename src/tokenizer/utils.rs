pub fn not_new_line(b: u8) -> bool {
    b != b'\n' && b != b'\r'
}

pub fn is_string_numeric(string: &str) -> bool {
    for c in string.chars() {
        if !c.is_numeric() {
            return false;
        }
    }
    return true;
}

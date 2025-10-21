const UNITS: [&str; 5] = ["B", "K", "M", "G", "T"];

/// Function convert bytes to a human readable format.
///
/// i.e.: 100 000 bytes = "97K"
pub fn bytes_to_human_readable(mut bytes: u64) -> String {
    let mut i = 0;

    while bytes >= 1024 && i < UNITS.len() - 1 {
        bytes /= 1024;
        i += 1;
    }

    format!("{}{}", bytes, UNITS[i])
}

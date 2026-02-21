//! Encoding support

pub mod utf8 {
    pub fn validate(_s: &str) -> bool {
        true
    }

    pub fn to_string(bytes: &[u8]) -> String {
        String::from_utf8_lossy(bytes).to_string()
    }
}

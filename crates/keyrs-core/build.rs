use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("key_codes.rs");
    let mut f = File::create(&dest_path).unwrap();

    // Generate the Key newtype wrapper
    writeln!(
        f,
        r#"
/// Represents a single keyboard key code.
///
/// This is a newtype wrapper around u16 for type safety.
/// The numeric values match Linux input-event-codes.h definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Key(pub u16);

impl Key {{
    /// Get the raw numeric code value
    pub fn code(self) -> u16 {{
        self.0
    }}

    /// Get the name of this key
    pub fn name(self) -> &'static str {{
        key_name(self.0)
    }}
}}

impl From<u16> for Key {{
    fn from(code: u16) -> Self {{
        Key(code)
    }}
}}

impl From<Key> for u16 {{
    fn from(key: Key) -> Self {{
        key.0
    }}
}}

impl fmt::Display for Key {{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{
        write!(f, "{{}}", self.name())
    }}
}}

impl FromStr for Key {{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {{
        key_from_name(s).ok_or_else(|| format!("Unknown key: {{}}", s))
    }}
}}
"#
    )
    .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}

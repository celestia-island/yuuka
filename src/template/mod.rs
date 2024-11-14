pub(crate) mod auto_enums;
pub(crate) mod auto_structs;
pub(crate) mod enums;
pub(crate) mod structs;

pub(crate) use auto_enums::generate_enums_auto_macros;
pub(crate) use auto_structs::generate_structs_auto_macros;
pub(crate) use enums::generate_enums_quote;
pub(crate) use structs::generate_structs_quote;

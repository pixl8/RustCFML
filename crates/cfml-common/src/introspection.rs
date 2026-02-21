//! Introspection support

use crate::dynamic::CfmlValue;

pub fn get_type(value: &CfmlValue) -> &'static str {
    value.type_name()
}

pub fn is_array(value: &CfmlValue) -> bool {
    matches!(value, CfmlValue::Array(_))
}

pub fn is_struct(value: &CfmlValue) -> bool {
    matches!(value, CfmlValue::Struct(_))
}

pub fn is_null(value: &CfmlValue) -> bool {
    matches!(value, CfmlValue::Null)
}

pub fn is_closure(value: &CfmlValue) -> bool {
    matches!(value, CfmlValue::Closure(_))
}

pub fn is_query(value: &CfmlValue) -> bool {
    matches!(value, CfmlValue::Query(_))
}

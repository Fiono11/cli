pub mod sign;
pub mod generate;
pub mod submit;
pub mod aggregate;

use scale_value::{Composite, ValueDef};

pub(crate) fn value_into_composite(value: scale_value::Value) -> scale_value::Composite<()> {
	match value.value {
		ValueDef::Composite(composite) => composite,
		_ => Composite::Unnamed(vec![value]),
	}
}
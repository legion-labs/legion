#[rustfmt::skip]
use super::layout_a::LayoutA;
use lgn_graphics_cgen_runtime::prelude::*;

pub struct LayoutB {
	pub(crate) a: Float3,
	pub(crate) b: Float4,
	pub(crate) c: LayoutA,
} // LayoutB


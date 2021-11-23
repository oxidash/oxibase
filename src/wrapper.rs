use crate::Ptr;
use crate::backend;
use crate::Address;
use crate::callconv::CallConv;

pub trait WrapperFunc<const F: Ptr> {
	/// The function type with no calling convention.
	type NoCCFunc;
	type CallConv: CallConv;

	fn real_addr() -> Address;
}

#[macro_export]
macro_rules! WrapperFuncInfo {
	($f:expr => $item:ident) => (<() as WrapperFunc<{ $crate::Ptr($f as *const ()) }>>::$item);
}

pub trait ResolveFnBase {
	/// Note: This will be called during init time, when some
	/// stdlib functions are broken, such as printing.
	fn resolve_base(self) -> Address;
}

impl ResolveFnBase for Address {
	fn resolve_base(self) -> Address {
		self
	}
}

impl<'a> ResolveFnBase for &'a str {
	fn resolve_base(self) -> Address {
		backend::global().get_module_base(self)
	}
}

pub trait ResolveFnOffset {
	/// Resolve an offset from a base.
	/// 
	/// Note: This will be called during init time, when some
	/// stdlib functions are broken, such as printing.
	/// # Safety
	/// The base must be a valid module.
	unsafe fn resolve_offset(self, base: Address) -> Address;
}

impl<'a> ResolveFnOffset for &'a str {
	unsafe fn resolve_offset(self, base: Address) -> Address {
		backend::global().get_symbol_address(base, self)
	}
}

impl ResolveFnOffset for u8 {
	unsafe fn resolve_offset(self, base: Address) -> Address {
		base + self as usize
	}
}

impl ResolveFnOffset for u16 {
	unsafe fn resolve_offset(self, base: Address) -> Address {
		base + self as usize
	}
}

impl ResolveFnOffset for u32 {
	unsafe fn resolve_offset(self, base: Address) -> Address {
		base + self as usize
	}
}

impl ResolveFnOffset for u64 {
	unsafe fn resolve_offset(self, base: Address) -> Address {
		base + self as usize
	}
}

impl ResolveFnOffset for usize {
	unsafe fn resolve_offset(self, base: Address) -> Address {
		base + self as usize
	}
}

impl ResolveFnOffset for i8 {
	unsafe fn resolve_offset(self, base: Address) -> Address {
		(base as isize + self as isize) as Address
	}
}

impl ResolveFnOffset for i16 {
	unsafe fn resolve_offset(self, base: Address) -> Address {
		(base as isize + self as isize) as Address
	}
}

impl ResolveFnOffset for i32 {
	unsafe fn resolve_offset(self, base: Address) -> Address {
		(base as isize + self as isize) as Address
	}
}

impl ResolveFnOffset for i64 {
	unsafe fn resolve_offset(self, base: Address) -> Address {
		(base as isize + self as isize) as Address
	}
}

impl ResolveFnOffset for isize {
	unsafe fn resolve_offset(self, base: Address) -> Address {
		(base as isize + self) as Address
	}
}
#![allow(incomplete_features)] // adt_const_params
#![feature(
	abi_thiscall,
	abi_vectorcall,
	const_maybe_uninit_assume_init,
	const_raw_ptr_deref,
	const_fn_fn_ptr_basics,
	const_fn_trait_bound,
	const_mut_refs,
	unboxed_closures,
	fn_traits,
	adt_const_params,
	asm,
	naked_functions
)]

/// Not part of the public API.
#[doc(hidden)]
pub mod __private {
	pub use ctor;
	//pub use oxibase_proc_macro;
	pub use paste;

	pub use Box;
}

#[macro_export]
macro_rules! raw_fns {
	($($tt:tt)*) => {
		$crate::__private::oxibase_proc_macro::raw_fns!($crate => $($tt)*);
	};
}

#[macro_use]
extern crate lazy_static;

pub mod backend;
pub mod callconv;
pub mod hook;
pub mod wrapper;

pub type Address = usize;

/// Pointer wrapper for const generics.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Ptr(pub *const ());

//#[cfg(test)]
pub mod tests {
	use crate::callconv::{CallConv, Cdecl};
	use crate::wrapper::WrapperFunc;

	use crate::select_cc;
	use crate::{Address, Ptr, WrapperFuncInfo};

	//#[test]
	pub fn manual_hook() {
		extern "cdecl" fn func_inner() -> u8 {
			4
		}

		fn func() -> u8 {
			func_inner()
		}

		impl WrapperFunc<{ Ptr(func as *const ()) }> for () {
			type NoCCFunc = fn() -> u8;
			type CallConv = Cdecl<(), u8>;

			fn real_addr() -> Address {
				func_inner as Address
			}
		}

		fn hook() -> u8 {
			5
		}

		const HOOK_T_CC: Ptr = Ptr(select_cc!(fn() -> u8 => extern (<WrapperFuncInfo!(func => CallConv)>::TRANSLATE_CC) = hook));

		let hook_t = <WrapperFuncInfo!(func => CallConv)>::get_translate::<{ HOOK_T_CC }>();
		let hooked = WrapperFuncInfo!(func => real_addr)();
		println!("{:p} {:p} {:p}", hook_t, HOOK_T_CC.0, hook as *const ());
		println!("hooking {:x}", hooked);
		assert_eq!(hooked, func_inner as _);
		unsafe {
			crate::backend::global().hook(hooked, hook_t as _);
		}
		println!("hooked");
		assert_eq!(func(), 5);
	}
}

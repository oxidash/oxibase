use std::marker::PhantomData;

use crate::Ptr;

macro_rules! standard_calling_convs {
	($d:tt $($v:ident = $cc:literal;)*) => {
		/// Standard calling conventions supported in Rust itself.
		#[derive(Debug, Clone, Copy, PartialEq, Eq)]
		pub enum StandardCallConv {
			$($v),*
		}

		$(
			pub struct $v<A, O>(PhantomData<fn(A) -> O>);

			impl<A, O> CallConv for $v<A, O> {
				type Args = A;
				type Output = O;

				const CALL_CC: StandardCallConv = StandardCallConv::$v;
				const TRANSLATE_CC: StandardCallConv = StandardCallConv::$v;

				fn get_call<const F: Ptr>() -> *const () {
					F.0
				}

				fn get_translate<const F: Ptr>() -> *const () {
					F.0
				}
			}
		)*

		#[doc(hidden)]
		#[macro_export]
		macro_rules! _change_cc {
			$(
				(extern $cc fn($d ($d argname:ident: $d argtype:ty),* $d (,)?) $d (-> $d ret:ty)? => extern $v = $d f:expr) => ({
					let func_with_cc_result: extern $cc fn($d ($d argtype),*) $d (-> $d ret)? = $f;
					func_with_cc_result
				});
				($d (extern $d from:literal)? fn($d ($d argname:ident: $d argtype:ty),* $d (,)?) $d (-> $d ret:ty)? => extern $v = $d f:expr) => ({
					extern $cc fn func_with_cc_result($d ($d argname: $d argtype)*) $d (-> $d ret)? {
						$d f($d ($d argname),*)
					}
					func_with_cc_result
				});
				(unsafe extern $cc fn($d ($d argname:ident: $d argtype:ty),* $d (,)?) $d (-> $d ret:ty)? => extern $v = $d f:expr) => ({
					let func_with_cc_result: unsafe extern $cc fn($d ($d argtype),*) $d (-> $d ret)? = $f;
					func_with_cc_result
				});
				(unsafe $d (extern $d from:literal)? fn($d ($d argname:ident: $d argtype:ty),* $d (,)?) $d (-> $d ret:ty)? => extern $v = $d f:expr) => ({
					unsafe extern $cc fn func_with_cc_result($d ($d argname: $d argtype)*) $d (-> $d ret)? {
						$d f($d ($d argname),*)
					}
					func_with_cc_result
				});
			)*
		}

		#[doc(hidden)]
		#[macro_export]
		macro_rules! _select_cc {
			($d (extern $d from:literal)? fn($d ($d argname:ident: $d argtype:ty),* $d (,)?) $d (-> $d ret:ty)? => extern ($d ex:expr) = $d f:expr) => ({
				let e: $crate::callconv::StandardCallConv = $d ex;
				match e {
					$(
						$crate::callconv::StandardCallConv::$v => $crate::change_cc!($d (extern $d from)? fn($d ($d argname: $d argtype),*) $d (-> $d ret)? => extern $v = $f) as *const ()
					),*
				}
			});
			(unsafe $d (extern $d from:literal)? fn($d ($d argname:ident: $d argtype:ty),* $d (,)?) $d (-> $d ret:ty)? => extern ($d ex:expr) = $d f:expr) => ({
				match $d ex {
					$(
						$crate::callconv::StandardCallConv::$v => $crate::change_cc!(unsafe $d (extern $d from)? fn($d ($d argname: $d argtype),*) $d (-> $d ret)? => extern $v = $f) as *const ()
					),*
				}
			});
		}
	};
}

#[cfg(all(target_arch = "x86", not(feature = "all_ccs")))]
standard_calling_convs! { $
	Cdecl = "cdecl";
	Stdcall = "stdcall";
	Thiscall = "thiscall";
	Fastcall = "fastcall";
	Vectorcall = "vectorcall";
}

#[cfg(all(target_arch = "x86_64", not(feature = "all_ccs")))]
standard_calling_convs! { $
	Cdecl = "cdecl";
	Win64 = "win64";
	Sysv64 = "sysv64";
	Vectorcall = "vectorcall";
}

#[cfg(feature = "all_ccs")]
standard_calling_convs! { $
	Cdecl = "cdecl";
	
	Stdcall = "stdcall";
	Thiscall = "thiscall";
	Fastcall = "fastcall";

	Win64 = "win64";
	Sysv64 = "sysv64";
	Vectorcall = "vectorcall";
}

/// Modify the calling convention of a function. If the
/// function already has the calling convention specified,
/// it is returned as-is. Otherwise, a wrapper is created.
/// 
/// # Example
/// ```
/// use oxibase::change_cc;
/// 
/// use std::mem;
/// 
/// extern "cdecl" fn double(x: u8) -> u8 {
/// 	x * 2
/// }
/// 
/// fn main() {
/// 	// no-op, since `double` is already cdecl
/// 	let double_cdecl = change_cc!(extern "cdecl" fn(x: u8) -> u8 => extern Cdecl = double);
/// 	assert_eq!(double_cdecl(4), 8);
/// }
/// ```
#[macro_export]
macro_rules! change_cc {
	($($tt:tt)*) => ($crate::callconv::__change_cc!($($tt)*));
}

/// Select a calling convention for the specified function based
/// on the value provided. Due to the fact that multiple different
/// calling conventions could be returned, this macro instead
/// returns a value of type `*const ()`.
/// 
/// # Example
/// ```
/// # #![feature(abi_vectorcall)]
/// use oxibase::callconv::StandardCallConv;
/// use oxibase::select_cc;
/// 
/// use std::mem;
/// 
/// extern "cdecl" fn double(x: u8) -> u8 {
/// 	x * 2
/// }
/// 
/// fn main() {
/// 	let s = StandardCallConv::Cdecl;
/// 
/// 	// no-op, since `double` is already cdecl
/// 	let double_ptr = select_cc!(extern "cdecl" fn(x: u8) -> u8 => extern (s) = double);
/// 
/// 	let double_cdecl: extern "cdecl" fn(u8) -> u8 = unsafe { mem::transmute(double_ptr) };
/// 	assert_eq!(double_cdecl(4), 8);
/// }
/// ```
#[macro_export]
macro_rules! select_cc {
	($($tt:tt)*) => ($crate::callconv::__select_cc!($($tt)*));
}

pub use _change_cc as __change_cc;
pub use _select_cc as __select_cc;

pub trait CallConv {
	/// A tuple of the arguments.
	type Args;
	/// The return value.
	type Output;

	/// The calling convention that the `get_call` function's
	/// return value should be transmuted to.
	const CALL_CC: StandardCallConv;

	/// The calling convention that should be passed to
	/// the `get_translate` function's return value.
	const TRANSLATE_CC: StandardCallConv;

	/// Gets the call function. We use a getter to allow more
	/// freedom in declaring the function.
	/// The following docs are for the return value returned
	/// from this getter, which although it is not a function type, is
	/// in essense a function:
	///
	/// Call the function `F`. `F` must be a function with
	/// the calling convention represented, and the correct
	/// arguments and output as specified in the type parameters.
	///
	/// This function shouldn't be called directly, hence why it is represented
	/// as a raw pointer. Instead, it should be transmuted to the type
	/// `extern Self::CALL_CC fn(...Self::Args) -> Self::Output`,
	/// where `...Self::Args` is the expanded tuple representing
	/// the actual arguments.
	fn get_call<const F: Ptr>() -> *const ();

	/// Gets the translate function. We use a getter to allow more
	/// freedom in declaring the function.
	/// The following docs are for the return value returned
	/// from this getter, which although it is not a function type, is
	/// in essense a function:
	///
	/// Translates a call to this function to a standard calling convention.
	/// `F` must be a function with the calling convention `Self::TRANSLATE_CC`,
	/// and the correct arguments and output as specified in the type parameters.
	///
	/// This function shouldn't be called directly, hence why it is represented
	/// as a raw pointer. Instead, it should be given to whatever needs the
	/// calling convention represented by this implementation.
	fn get_translate<const F: Ptr>() -> *const ();
}

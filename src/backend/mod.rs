use std::mem::MaybeUninit;

use crate::Address;

pub trait Detour {
	fn target(&self) -> Address;
	fn hook(&self) -> Address;
	fn is_enabled(&self) -> bool;
	fn set_enabled(&self, b: bool);
	fn enable(&self) { self.set_enabled(true); }
	fn disable(&self) { self.set_enabled(false); }
}

impl<T: Detour + ?Sized> Detour for Box<T> {
	fn target(&self) -> Address {
		(&**self).target()
	}
	fn hook(&self) -> Address {
		(&**self).hook()
	}
	fn is_enabled(&self) -> bool { (&**self).is_enabled() }
	fn set_enabled(&self, b: bool) { (&**self).set_enabled(b); }
	fn enable(&self) { (&**self).enable(); }
	fn disable(&self) { (&**self).disable(); }
}

pub trait Backend {

	fn name(&self) -> &'static str { "(unnamed backend)" }

	fn get_module_base(&self, module: &str) -> Address;

	unsafe fn get_symbol_address(&self, base: Address, symbol: &str) -> Address;

	unsafe fn hook(&self, target: Address, detour: Address) -> Box<dyn Detour>;
}

static mut GLOBAL_BACKEND: &'static dyn Backend = &DefaultBackend::new();

pub fn global() -> &'static dyn Backend {
	// SAFETY: This will never cause a data race unless
	// `set_global` is called, but that function is unsafe
	unsafe { GLOBAL_BACKEND }
}

/// # Safety
/// This must happen before any access of the global backend.
pub unsafe fn set_global(x: &'static dyn Backend) {
	GLOBAL_BACKEND = x;
}

#[cfg(feature = "backend")]
mod impl_backend {
	use detour::RawDetour;

	use std::ffi::CString;

	use super::*;

	pub struct DefaultDetour {
		pub target: Address,
		pub detour: Address,
		pub raw: RawDetour
	}

	impl Detour for DefaultDetour {
		fn target(&self) -> Address { self.target }

		fn hook(&self) -> Address { self.detour }

		fn is_enabled(&self) -> bool { self.raw.is_enabled() }

		fn set_enabled(&self, b: bool) {
			unsafe {
				if b {
					self.raw.enable().unwrap();
				} else {
					self.raw.disable().unwrap();
				}
			}
		}
	}

	pub struct DefaultBackend(());

	impl DefaultBackend {
		pub const fn new() -> Self { Self(()) }
	}

	impl Backend for DefaultBackend {
		fn name(&self) -> &'static str { "default oxibase backend impl" }

		fn get_module_base(&self, name: &str) -> Address {
			let cs = if let Ok(x) = CString::new(name) {
				x
			} else {
				return 0;
			};
			unsafe {
				#[cfg(windows)]
				{
					platform_lib::um::libloaderapi::GetModuleHandleA(cs.as_ptr());
				}
				#[cfg(not(windows))]
				{
					platform_lib::dlopen(cs.as_ptr(), platform_lib::RTLD_NOLOAD) as _
				}
			}
		}

		unsafe fn get_symbol_address(&self, module: Address, sym: &str) -> Address {
			let cs = if let Ok(x) = CString::new(sym) {
				x
			} else {
				return 0;
			};
			#[cfg(windows)]
			{
				platform_lib::um::libloaderapi::GetProcAddress(module, cs.as_ptr())
			}
			#[cfg(not(windows))]
			{
				platform_lib::dlsym(module as _, cs.as_ptr()) as _
			}
		}

		unsafe fn hook(&self, target: Address, detour: Address) -> Box<dyn Detour> {
			let raw = RawDetour::new(target as _, detour as _).unwrap();
			let d = DefaultDetour { target, detour, raw };
			d.enable();
			Box::new(d)
		}
	}
}

#[cfg(not(feature = "backend"))]
mod impl_backend {
	pub struct DefaultBackend(());

	impl DefaultBackend {
		pub const fn new() -> Self { Self(()) }
	}

	impl Backend for DefaultBackend {
		fn name(&self) -> &'static str {
			panic!("backend not initialized!")
		}

		fn get_module_base(&self, name: &str) -> Address {
			panic!("backend not initialized!")
		}

		unsafe fn get_symbol_address(&self, module: Address, sym: &str) -> Address {
			panic!("backend not initialized!")
		}

		unsafe fn hook(&self, target: Address, detour: Address) -> Box<dyn Detour> {
			panic!("backend not initialized!")
		}
	}
}

pub use impl_backend::DefaultBackend;
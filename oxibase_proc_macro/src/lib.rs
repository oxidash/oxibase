#![cfg(any())]
#![feature(build_hasher_simple_hash_one)]

extern crate proc_macro;

use quote::{format_ident, quote, quote_spanned};

use syn::*;

use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;

use std::hash::BuildHasher;

use std::collections::hash_map::RandomState;
use std::time::{SystemTime, UNIX_EPOCH};

use proc_macro::TokenStream;

#[allow(dead_code)]
struct RawFnsInput {
	krate: Ident,
	arrow: Token![=>],
	base: Option<Lit>,
	fns: Vec<RawFn>,
}

impl Parse for RawFnsInput {
	fn parse(input: ParseStream) -> syn::parse::Result<Self> {
		let krate = input.parse()?;
		let arrow = input.parse()?;
		let base = input.parse()?;
		let mut fns = Vec::new();
		while !input.is_empty() {
			let f = input.parse()?;
			fns.push(f);
		}
		Ok(RawFnsInput {
			krate,
			arrow,
			base,
			fns,
		})
	}
}

#[allow(dead_code)]
struct RawFn {
	vis: Visibility,
	sig: Signature,
	body: RawFnBody,
}

impl Parse for RawFn {
	fn parse(input: ParseStream) -> syn::parse::Result<Self> {
		Ok(RawFn {
			vis: input.parse()?,
			sig: input.parse()?,
			body: input.parse()?,
		})
	}
}

enum RawFnBody {
	Offset(RawFnBodyOffset),
	Custom(Block),
}

impl Parse for RawFnBody {
	fn parse(input: ParseStream) -> Result<Self> {
		let l = input.lookahead1();
		match () {
			_ if l.peek(Token![=]) => Ok(Self::Offset(input.parse()?)),
			_ if l.peek(token::Brace) => Ok(Self::Custom(input.parse()?)),
			_ => Err(l.error()),
		}
	}
}

struct RawFnBodyOffset {
	eq: Token![=],
	abi: Option<Abi>,
	offset: Expr,
	semi: Token![;],
}

impl Parse for RawFnBodyOffset {
	fn parse(input: ParseStream) -> Result<Self> {
		Ok(Self {
			eq: input.parse()?,
			abi: input.parse()?,
			offset: input.parse()?,
			semi: input.parse()?,
		})
	}
}

#[allow(nonstandard_style)]
fn fn_pointer_of(sig: &Signature) -> Type {
	let Signature {
		constness: _,
		asyncness,
		unsafety: unsafe_,
		abi: extern_,
		fn_token: fn_,
		ident: _,
		generics,
		inputs: fn_args,
		output: Ret @ _,
		..
	} = sig;
	assert!(asyncness.is_none());
	let each_lifetime = generics.lifetimes().map(|it| &it.lifetime);
	let EachArg @ _ = fn_args.iter().map(|it| match it {
		FnArg::Receiver(Receiver {
			reference,
			mutability: mut_,
			self_token: self_,
			..
		}) => {
			let Self_ @ _ = Ident::new("Self", self_.span);
			let ref_ = reference.as_ref().map(|(and, lt)| quote!(#and #lt #mut_));
			quote!( #ref_ #Self_ )
		}
		FnArg::Typed(PatType { ty, .. }) => quote!( #ty ),
	});
	parse_quote!(
		for<#(#each_lifetime),*>
		#unsafe_
		#extern_
		#fn_ ( #(#EachArg),* ) #Ret
	)
}

#[proc_macro]
pub fn raw_fns(input: TokenStream) -> TokenStream {
	let hash = RandomState::new().hash_one(input.to_string());

	let RawFnsInput {
		krate, base, fns, ..
	} = parse_macro_input!(input as RawFnsInput);

	let fns_len = fns.len();
	let register = format_ident!(
		"__oxibase_{}_{:x}_{}_raw_fns_register",
		fns_len,
		hash,
		SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap()
			.as_nanos()
	);

	let mut output = TokenStream::new();

	let mut values = Vec::new();

	for (
		index,
		RawFn {
			vis,
			sig,
			body,
		},
	) in fns.into_iter().enumerate()
	{
		let name = &sig.ident;
		//let fptr = fn_pointer_of(&sig);
		
		output.extend(TokenStream::from(quote! {
			#vis #sig {

			}
		}));
	}

	let base_span = base.span();

	
	output
}

use proc_macro::{TokenStream};
use proc_macro2::{Ident, Span};
use syn::{parse_macro_input, Item, Meta, Expr, Lit, ExprLit, MetaNameValue, ItemStruct, Path, Type, GenericArgument, TypePath, Token, PathSegment, parenthesized, LifetimeParam, ItemEnum, Generics, WhereClause, LitStr};
use quote::{quote, quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::PathArguments::AngleBracketed;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::{PathSep};

struct TableField {
	fso_name: String,
	rust_token: Ident,
	rust_type: Type,
	rust_span: Span
}

struct EnumField {
	fso_name: String,
	rust_token: Ident,
	rust_contents: Vec<(Ident, Type)>,
	rust_span: Span
}

fn fso_table_build_parse(fields: &Vec<TableField>) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
	let mut parse = proc_macro2::TokenStream::new();
	let mut fill = proc_macro2::TokenStream::new();
	
	for field in fields.iter() {
		let name = &field.rust_token;
		
		let new_parse = match &field.rust_type {
			Type::Path( TypePath { path: Path { segments, .. }, ..} ) if segments.last().map_or(false, |outer_type| outer_type.ident == "Option") => {
				let angle_brackets = 
				if let AngleBracketed(inner_types) = &segments.last().unwrap().arguments {
					inner_types.args.first().unwrap()
				}
				else {
					panic!("Unparametrized Option found!");
				};
				if let GenericArgument::Type( inner_type ) = &angle_brackets {
					quote!(
						let #name = <#inner_type as FSOTable<Parser>>::parse(state)?;
					)
				}
				else {
					panic!("Unparametrized Option found!");
				}
			}
			Type::Path( TypePath { path: Path { segments, .. }, ..} ) if segments.last().map_or(false, |outer_type| outer_type.ident == "Vec") => {
				let angle_brackets =
					if let AngleBracketed(inner_types) = &segments.last().unwrap().arguments {
						inner_types.args.first().unwrap()
					}
					else {
						panic!("Unparametrized Vec found!");
					};
				if let GenericArgument::Type( Type::Path( TypePath { path , .. } ) ) = &angle_brackets {
					let inner_type = &path.segments.last().unwrap().ident;
					quote!(
						let mut #name = Vec::default();
						while let Ok(__new_element_for_vec) = <#inner_type as FSOTable<Parser>>::parse(state) {
							#name.push(__new_element_for_vec);
						}
					)
				}
				else {
					panic!("Unparametrized Vec found!");
				}
			}
			Type::Path( TypePath { path, ..} ) => {
				quote!(
					let #name = <#path as FSOTable<Parser>>::parse(state)?;
				)
			}
			_ => {
				quote_spanned! {
					field.rust_span =>
					compile_error!("Cannot process non-path types for FSO table parsing!");
				}
			}
		};
		
		parse = quote!(
			#parse
			#new_parse
		);
		
		fill = quote!(
			#fill
			#name,
		);
	}

	(parse, fill)
}

fn fso_enum_build_variant_parse(ident: &Ident, field_type: &Type) -> proc_macro2::TokenStream {
	quote! {
		#ident: { <#field_type as FSOTable<Parser>>::parse(state)? }
	}
}

fn fso_build_impl_generics(lifetime_req: &Vec<proc_macro2::TokenStream>, generics: &Generics) -> proc_macro2::TokenStream {
	let mut impl_with_generics = proc_macro2::TokenStream::new();
	for lifetime in lifetime_req.iter() {
		impl_with_generics = quote! {#impl_with_generics #lifetime, };
	}

	if !generics.params.is_empty() && lifetime_req.is_empty() {
		let inner_generics = generics.params.to_token_stream();
		impl_with_generics = quote! {#impl_with_generics #inner_generics, };
	}

	quote!{#impl_with_generics 'parser, Parser}
}

fn fso_build_where_clause(instancing_req: &Vec<proc_macro2::TokenStream>, where_clause: &Option<&WhereClause>) -> proc_macro2::TokenStream {
	let mut where_clause_with_parser = if let Some(where_clause) = where_clause {
		let inner_where = where_clause.to_token_stream();
		quote! {#inner_where, }
	}
	else {
		quote! {where }
	};
	
	for instancing_type in instancing_req.iter() {
		where_clause_with_parser = quote! {#where_clause_with_parser Parser: #instancing_type, };
	}

	where_clause_with_parser
}

fn fso_table_struct(item_struct: &mut ItemStruct, instancing_req: Vec<proc_macro2::TokenStream>, lifetime_req: Vec<proc_macro2::TokenStream>) -> proc_macro2::TokenStream {
	let mut table_fields: Vec<TableField> = Vec::new();
	let struct_name = &item_struct.ident;
	let (_, ty_generics, where_clause) = item_struct.generics.split_for_impl();
	
	if let syn::Fields::Named(ref mut fields) = item_struct.fields {
		for field in fields.named.iter_mut() {
			let rust_type = field.ty.clone();
			let forced_table_name = field.attrs.iter().find_map(|a| match &a.meta {
				Meta::NameValue( MetaNameValue { value: Expr::Lit( ExprLit{ lit: Lit::Str(new_name), ..}), path, .. })
				if path.is_ident("fso_name") => { Some(new_name.value()) },
				Meta::NameValue(MetaNameValue { path, .. }) if path.is_ident("fso_name") => {
					//TODO error here
					None
				}
				_ => { None }
			});
			let skip = field.attrs.iter().find_map(|a| match &a.meta {
				Meta::Path( path ) if path.is_ident("skip") => {
					Some(())
				}
				_ => { None }
			});
			field.attrs.retain(|a| !(a.path().is_ident("fso_name") || a.path().is_ident("skip")));

			if skip.is_some() {
				continue;
			}
			
			if let Some(ident) = field.ident.as_ref() {
				let rust_token = ident.to_string();
				let fso_name = forced_table_name.unwrap_or("$".to_string() + &rust_token[..1].to_string().to_uppercase() + &rust_token[1..] + ":");

				table_fields.push(TableField { fso_name, rust_token: ident.clone(), rust_type, rust_span: field.span() });
			}
		}
		/*fields.named.push(
			syn::Field::parse_named
				.parse2(quote! { __unknown_fso_fields: Vec<String> })
				.unwrap(),
		);*/
	}
	else {
		panic!("Could not add fields to table struct!");
	}
	
	let impl_with_generics = fso_build_impl_generics(&lifetime_req, &item_struct.generics);
	
	let where_clause_with_parser = fso_build_where_clause(&instancing_req, &where_clause);
	
	let (parser, filler) = fso_table_build_parse(&table_fields);
	
	quote! {
		impl <#impl_with_generics> fso_tables::FSOTable<'parser, Parser> for #struct_name #ty_generics #where_clause_with_parser {
			fn parse(state: &'parser Parser) -> Result<#struct_name #ty_generics, fso_tables::FSOParsingError> {
				#parser
				core::result::Result::Ok(#struct_name {
					#filler
				})
			}
			fn dump(&self) { }
		}
	}
}

fn fso_table_enum(item_enum: &mut ItemEnum, instancing_req: Vec<proc_macro2::TokenStream>, lifetime_req: Vec<proc_macro2::TokenStream>, prefix: String, suffix: String) -> proc_macro2::TokenStream {
	let struct_name = &item_enum.ident;
	let (_, ty_generics, where_clause) = item_enum.generics.split_for_impl();
	
	let mut parser = quote!();
	let mut fail_message = "Expected one of ".to_string();
	
	for option in &item_enum.variants{
		let fso_name = format!("{}{}{}", prefix, option.ident, suffix);
		fail_message = format!("{}{}, ", fail_message, fso_name);
		
		let mut field_parsers = quote!();
		for field in &option.fields{
			if let Some(ident) = &field.ident {
				let field_parser = fso_enum_build_variant_parse(ident, &field.ty);
				field_parsers = quote! {
					#field_parsers
					#field_parser,
				};
			}
			else {
				return quote_spanned! {
					field.span() => FSO table enums cannot have unnamed fields.
				};
			}
		}
		
		let rust_name = &option.ident;
		
		parser = quote! {
			#parser
			if let Ok(_) = state.consume_string(#fso_name) {
				return Ok( #struct_name::#rust_name {
					#field_parsers
				});
			}
		}
	}

	fail_message = format!("{}got {{}}.", fail_message);
	
	let impl_with_generics = fso_build_impl_generics(&lifetime_req, &item_enum.generics);

	let where_clause_with_parser = fso_build_where_clause(&instancing_req, &where_clause);
	
	quote! {
		impl <#impl_with_generics> fso_tables::FSOTable<'parser, Parser> for #struct_name #ty_generics #where_clause_with_parser {
			fn parse(state: &'parser Parser) -> Result<#struct_name #ty_generics, fso_tables::FSOParsingError> {
				#parser
				let current = state.get();
				let current_cut = &current[..std::cmp::min(20, current.len())];
				core::result::Result::Err(fso_tables::FSOParsingError { reason: format!(#fail_message, current_cut), line: state.line() })
			}
			fn dump(&self) { }
		}
	}
}

#[proc_macro_attribute]
pub fn fso_table(args: TokenStream, input: TokenStream) -> TokenStream  {
	let mut pre_item_out = proc_macro2::TokenStream::new();
	let mut item = parse_macro_input!(input as Item);
	let mut post_item_out = proc_macro2::TokenStream::new();

	let mut required_parser_traits: Vec<proc_macro2::TokenStream> = vec![quote!(FSOParser<'parser>)];
	let mut required_lifetimes: Vec<proc_macro2::TokenStream> = vec![];
	
	let mut enum_prefix = "".to_string();
	let mut enum_suffix = "".to_string();
	
	struct ReqTraitParser {
		data: Punctuated::<PathSegment, PathSep>
	}
	impl Parse for ReqTraitParser{
		fn parse(tokens: ParseStream) -> syn::Result<ReqTraitParser> {
			let parser = Punctuated::<PathSegment, PathSep>::parse_separated_nonempty;
			let result = ReqTraitParser{ data: parser(tokens)? };
			Ok(result)
		}
	}
	
	let args_parser = syn::meta::parser(|meta| {
		if meta.path.is_ident("required_parser_trait") {
			let content;
			parenthesized!(content in meta.input);
			
			for req_trait in &content.parse_terminated(ReqTraitParser::parse, Token![,])? {
				let trait_path = Path { leading_colon: None, segments: req_trait.data.clone() };
				required_parser_traits.push(quote!(#trait_path));
			};
			Ok(())
		}
		else if meta.path.is_ident("required_lifetime") {
			let content;
			parenthesized!(content in meta.input);

			for lifetime in &content.parse_terminated(LifetimeParam::parse, Token![,])? {
				required_lifetimes.push(quote!(#lifetime));
			};
			Ok(())
		}
		else if meta.path.is_ident("prefix") {
			enum_prefix = meta.value()?.parse::<LitStr>()?.value();
			Ok(())
		}
		else if meta.path.is_ident("suffix") {
			enum_suffix = meta.value()?.parse::<LitStr>()?.value();
			Ok(())
		}
		else {
			Err(meta.error("Unsupported FSO table property"))
		}
	});
	parse_macro_input!(args with args_parser);

	match &mut item {
		Item::Struct(item_struct) => {
			post_item_out = fso_table_struct(item_struct, required_parser_traits, required_lifetimes);
		}
		Item::Enum(item_enum) => {
			post_item_out = fso_table_enum(item_enum, required_parser_traits, required_lifetimes, enum_prefix, enum_suffix);
		}
		_ => {
			pre_item_out = quote_spanned! {
                item.span() =>
                compile_error!("Can only annotate structs!");
            };
		}
	}

	return quote! {
        #pre_item_out
        #item
        #post_item_out
    }.into();
}
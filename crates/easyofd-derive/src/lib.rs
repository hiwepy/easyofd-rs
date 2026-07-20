//! Derive macros for easyofd-rs.
//!
//! Provides `#[derive(OfdModel)]` for automatic `OfdModel` trait implementation.

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Derive macro for the `OfdModel` trait.
///
/// # Struct-level attributes
///
/// - `#[ofd(page_width = 210.0, page_height = 297.0)]` — page dimensions in mm
///
/// # Field-level attributes
///
/// - `#[ofd(x = 20.0, y = 30.0)]` — position in mm (required for each field)
/// - `#[ofd(font = "SimSun")]` — font family (default: "`SimSun`")
/// - `#[ofd(size = 12.0)]` — font size in pt (default: 12.0)
/// - `#[ofd(weight = 400)]` — font weight (default: 400)
/// - `#[ofd(bold)]` — shorthand for weight 700
/// - `#[ofd(italic)]` — italic text
/// - `#[ofd(color = 0)]` — text color as RGB hex (default: 0 = black)
/// - `#[ofd(kind = "image")]` — render as image instead of text
/// - `#[ofd(ignore)]` — skip this field entirely
///
/// # Example
///
/// ```rust,ignore
/// use easyofd::OfdModel;
///
/// #[derive(OfdModel)]
/// #[ofd(page_width = 210.0, page_height = 297.0)]
/// struct Invoice {
///     #[ofd(x = 20.0, y = 30.0, size = 18.0, bold)]
///     title: String,
///     #[ofd(x = 20.0, y = 50.0)]
///     amount: String,
///     #[ofd(x = 150.0, y = 30.0, x = 30.0, y = 30.0, kind = "image")]
///     seal: Vec<u8>,
/// }
/// ```
#[proc_macro_derive(OfdModel, attributes(ofd))]
pub fn derive_ofd_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_ofd_model(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn impl_ofd_model(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let (page_width, page_height) = parse_page_attrs(input)?;

    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            input,
            "OfdModel can only be derived for structs",
        ));
    };

    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(
            input,
            "OfdModel can only be derived for structs with named fields",
        ));
    };

    let (schema_entries, page_pushes) = process_fields(&fields.named)?;

    let schema_len = schema_entries.len();

    let expanded = quote! {
        impl #impl_generics easyofd_core::OfdModel for #name #type_generics #where_clause {
            fn schema() -> &'static [easyofd_core::OfdField] {
                static SCHEMA: std::sync::OnceLock<Vec<easyofd_core::OfdField>> = std::sync::OnceLock::new();
                SCHEMA.get_or_init(|| vec![#(#schema_entries),*]).as_slice()
            }

            fn page_size() -> (f64, f64) {
                (#page_width, #page_height)
            }

            fn to_page(&self) -> easyofd_core::OfdResult<easyofd_core::OfdPage> {
                let (width, height) = Self::page_size();
                let mut content = Vec::with_capacity(#schema_len);
                #(#page_pushes)*
                Ok(easyofd_core::OfdPage { width, height, content })
            }
        }
    };

    Ok(expanded)
}

fn process_fields(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::Token![,]>,
) -> syn::Result<(Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>)> {
    let mut schema_entries = Vec::new();
    let mut page_pushes = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        if has_ignore_attr(field) {
            continue;
        }

        let cfg = parse_field_attrs(field)?;
        let fx = cfg.x;
        let fy = cfg.y;
        let font = &cfg.font;
        let fsize = cfg.size;
        let fweight = cfg.weight;
        let fitalic = cfg.italic;
        let fcolor = cfg.color;

        let is_image = cfg.kind == "image";

        if is_image {
            schema_entries.push(quote! {
                easyofd_core::OfdField {
                    name: #field_name_str,
                    position: (#fx, #fy),
                    font: #font,
                    size: #fsize,
                    weight: #fweight,
                    italic: #fitalic,
                    color: #fcolor,
                    kind: easyofd_core::OfdFieldKind::Image,
                }
            });
            let img_w = cfg.img_width;
            let img_h = cfg.img_height;
            page_pushes.push(quote! {
                content.push(easyofd_core::ContentObject::Image(
                    easyofd_core::ImageObject::jpeg(#fx, #fy, #img_w, #img_h, self.#field_name.clone())
                ));
            });
        }

        if !is_image {
            schema_entries.push(quote! {
                easyofd_core::OfdField {
                    name: #field_name_str,
                    position: (#fx, #fy),
                    font: #font,
                    size: #fsize,
                    weight: #fweight,
                    italic: #fitalic,
                    color: #fcolor,
                    kind: easyofd_core::OfdFieldKind::Text,
                }
            });
            page_pushes.push(quote! {
                content.push(easyofd_core::ContentObject::Text(
                    easyofd_core::TextObject::new(#fx, #fy, self.#field_name.to_string())
                        .font(#font)
                        .size(#fsize)
                        .color(#fcolor)
                ));
            });
        }
    }

    Ok((schema_entries, page_pushes))
}

struct FieldConfig {
    x: f64,
    y: f64,
    font: String,
    size: f64,
    weight: u32,
    italic: bool,
    color: u32,
    kind: String,
    img_width: f64,
    img_height: f64,
}

fn parse_page_attrs(input: &DeriveInput) -> syn::Result<(f64, f64)> {
    let mut width = 210.0_f64;
    let mut height = 297.0_f64;

    for attr in &input.attrs {
        if !attr.path().is_ident("ofd") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            let ident = meta.path.require_ident()?.to_string();
            if ident == "page_width" {
                let value = meta.value()?.parse::<syn::Lit>()?;
                width = parse_lit_f64(&value)?;
            }
            if ident == "page_height" {
                let value = meta.value()?.parse::<syn::Lit>()?;
                height = parse_lit_f64(&value)?;
            }
            Ok(())
        })?;
    }

    Ok((width, height))
}

fn has_ignore_attr(field: &syn::Field) -> bool {
    for attr in &field.attrs {
        if !attr.path().is_ident("ofd") {
            continue;
        }
        let mut ignore = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("ignore") {
                ignore = true;
            }
            Ok(())
        });
        if ignore {
            return true;
        }
    }
    false
}

fn parse_field_attrs(field: &syn::Field) -> syn::Result<FieldConfig> {
    let mut cfg = FieldConfig {
        x: 0.0,
        y: 0.0,
        font: "SimSun".to_string(),
        size: 12.0,
        weight: 400,
        italic: false,
        color: 0,
        kind: "text".to_string(),
        img_width: 30.0,
        img_height: 30.0,
    };

    for attr in &field.attrs {
        if !attr.path().is_ident("ofd") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            let ident = meta.path.require_ident()?.to_string();
            if ident == "x" {
                let value = meta.value()?.parse::<syn::Lit>()?;
                cfg.x = parse_lit_f64(&value)?;
            }
            if ident == "y" {
                let value = meta.value()?.parse::<syn::Lit>()?;
                cfg.y = parse_lit_f64(&value)?;
            }
            if ident == "font" {
                let value = meta.value()?.parse::<syn::Lit>()?;
                cfg.font = lit_to_string(&value);
            }
            if ident == "size" {
                let value = meta.value()?.parse::<syn::Lit>()?;
                cfg.size = parse_lit_f64(&value)?;
            }
            if ident == "weight" {
                let value = meta.value()?.parse::<syn::Lit>()?;
                cfg.weight = parse_lit_u32(&value)?;
            }
            if ident == "bold" {
                cfg.weight = 700;
            }
            if ident == "italic" {
                cfg.italic = true;
            }
            if ident == "color" {
                let value = meta.value()?.parse::<syn::Lit>()?;
                cfg.color = parse_lit_u32(&value)?;
            }
            if ident == "kind" {
                let value = meta.value()?.parse::<syn::Lit>()?;
                cfg.kind = lit_to_string(&value);
            }
            if ident == "img_width" {
                let value = meta.value()?.parse::<syn::Lit>()?;
                cfg.img_width = parse_lit_f64(&value)?;
            }
            if ident == "img_height" {
                let value = meta.value()?.parse::<syn::Lit>()?;
                cfg.img_height = parse_lit_f64(&value)?;
            }
            Ok(())
        })?;
    }

    Ok(cfg)
}

/// Parse a `Lit` as `f64` via string conversion — avoids `Lit` enum branching.
fn parse_lit_f64(lit: &syn::Lit) -> syn::Result<f64> {
    let s = lit.to_token_stream().to_string().replace('_', "");
    Ok(s.parse().unwrap_or(0.0))
}

/// Parse a `Lit` as `u32` via string→f64 conversion — avoids `Lit` enum branching.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn parse_lit_u32(lit: &syn::Lit) -> syn::Result<u32> {
    let s = lit.to_token_stream().to_string().replace('_', "");
    Ok(s.parse::<f64>().unwrap_or(0.0) as u32)
}

/// Extract the string value from a `Lit`, or empty string if not a string literal.
fn lit_to_string(lit: &syn::Lit) -> String {
    lit.to_token_stream().to_string()
        .trim_matches('"')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::Lit;

    fn lit_from_tokens(tokens: proc_macro2::TokenStream) -> Lit {
        syn::parse2(tokens).expect("failed to parse literal")
    }

    #[test]
    fn test_parse_lit_f64_float() {
        let lit = lit_from_tokens(quote::quote!(42.5));
        assert!((parse_lit_f64(&lit).unwrap() - 42.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_lit_f64_int() {
        let lit = lit_from_tokens(quote::quote!(42));
        assert!((parse_lit_f64(&lit).unwrap() - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_lit_f64_str() {
        // String literal should parse as 0.0 via unwrap_or fallback
        let lit = lit_from_tokens(quote::quote!("hello"));
        assert!((parse_lit_f64(&lit).unwrap() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_lit_u32_int() {
        let lit = lit_from_tokens(quote::quote!(42));
        assert_eq!(parse_lit_u32(&lit).unwrap(), 42);
    }

    #[test]
    fn test_parse_lit_u32_float() {
        let lit = lit_from_tokens(quote::quote!(42.5));
        assert_eq!(parse_lit_u32(&lit).unwrap(), 42);
    }

    #[test]
    fn test_parse_lit_u32_str() {
        // String literal should parse as 0 via unwrap_or fallback
        let lit = lit_from_tokens(quote::quote!("hello"));
        assert_eq!(parse_lit_u32(&lit).unwrap(), 0);
    }

    #[test]
    fn test_lit_to_string() {
        let lit = lit_from_tokens(quote::quote!("SimHei"));
        assert_eq!(lit_to_string(&lit), "SimHei");
    }

    #[test]
    fn test_impl_ofd_model_enum_errors() {
        // Derive on an enum should fail
        let input: DeriveInput = syn::parse_quote! {
            enum NotAStruct { A, B }
        };
        assert!(impl_ofd_model(&input).is_err());
    }

    #[test]
    fn test_impl_ofd_model_tuple_struct_errors() {
        // Derive on a tuple struct should fail
        let input: DeriveInput = syn::parse_quote! {
            struct TupleStruct(u8, String);
        };
        assert!(impl_ofd_model(&input).is_err());
    }
}

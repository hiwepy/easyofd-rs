//! Derive macros for easyofd-rs.
//!
//! Provides `#[derive(OfdModel)]` for automatic `OfdModel` trait implementation.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Lit, parse_macro_input};

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

        match cfg.kind.as_str() {
            "text" => {
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
            "image" => {
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
            other => {
                return Err(syn::Error::new_spanned(
                    field,
                    format!("unknown ofd kind: \"{other}\". Expected \"text\" or \"image\""),
                ));
            }
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
            match ident.as_str() {
                "page_width" => {
                    let value = meta.value()?.parse::<Lit>()?;
                    width = lit_to_f64(&value)?;
                }
                "page_height" => {
                    let value = meta.value()?.parse::<Lit>()?;
                    height = lit_to_f64(&value)?;
                }
                _ => {}
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
            match ident.as_str() {
                "x" => {
                    let value = meta.value()?.parse::<Lit>()?;
                    cfg.x = lit_to_f64(&value)?;
                }
                "y" => {
                    let value = meta.value()?.parse::<Lit>()?;
                    cfg.y = lit_to_f64(&value)?;
                }
                "font" => {
                    let value = meta.value()?.parse::<Lit>()?;
                    if let Lit::Str(s) = value {
                        cfg.font = s.value();
                    }
                }
                "size" => {
                    let value = meta.value()?.parse::<Lit>()?;
                    cfg.size = lit_to_f64(&value)?;
                }
                "weight" => {
                    let value = meta.value()?.parse::<Lit>()?;
                    cfg.weight = lit_to_u32(&value)?;
                }
                "bold" => cfg.weight = 700,
                "italic" => cfg.italic = true,
                "color" => {
                    let value = meta.value()?.parse::<Lit>()?;
                    cfg.color = lit_to_u32(&value)?;
                }
                "kind" => {
                    let value = meta.value()?.parse::<Lit>()?;
                    if let Lit::Str(s) = value {
                        cfg.kind = s.value();
                    }
                }
                "img_width" => {
                    let value = meta.value()?.parse::<Lit>()?;
                    cfg.img_width = lit_to_f64(&value)?;
                }
                "img_height" => {
                    let value = meta.value()?.parse::<Lit>()?;
                    cfg.img_height = lit_to_f64(&value)?;
                }
                _ => {}
            }
            Ok(())
        })?;
    }

    Ok(cfg)
}

fn lit_to_f64(lit: &Lit) -> syn::Result<f64> {
    // The Rust parser only emits Lit::Float or Lit::Int for numeric attribute values.
    // Lit::Str, Lit::Bool, Lit::Char etc. cannot appear here.
    if let Lit::Float(f) = lit {
        f.base10_parse()
    } else if let Lit::Int(i) = lit {
        Ok(f64::from(i.base10_parse::<i32>()?))
    } else {
        unreachable!("numeric attribute value is always Float or Int")
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn lit_to_u32(lit: &Lit) -> syn::Result<u32> {
    if let Lit::Int(i) = lit {
        i.base10_parse()
    } else if let Lit::Float(f) = lit {
        let val = f.base10_parse::<f64>()?;
        if val < 0.0 || val > f64::from(u32::MAX) {
            return Err(syn::Error::new_spanned(lit, "value out of u32 range"));
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        { Ok(val as u32) }
    } else {
        unreachable!("numeric attribute value is always Float or Int")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::Lit;

    fn lit_from_tokens(tokens: proc_macro2::TokenStream) -> Lit {
        syn::parse2(tokens).expect("failed to parse literal")
    }

    // ── lit_to_f64 ────────────────────────────────────────────────────────────

    #[test]
    fn test_lit_to_f64_float() {
        let lit = lit_from_tokens(quote::quote!(42.5));
        let result = lit_to_f64(&lit).unwrap();
        assert!((result - 42.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_lit_to_f64_int() {
        let lit = lit_from_tokens(quote::quote!(42));
        let result = lit_to_f64(&lit).unwrap();
        assert!((result - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    #[should_panic(expected = "numeric attribute value is always Float or Int")]
    fn test_lit_to_f64_unreachable() {
        let lit = lit_from_tokens(quote::quote!("hello"));
        let _ = lit_to_f64(&lit);
    }

    // ── lit_to_u32 ────────────────────────────────────────────────────────────

    #[test]
    fn test_lit_to_u32_int() {
        let lit = lit_from_tokens(quote::quote!(42));
        let result = lit_to_u32(&lit).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_lit_to_u32_float() {
        let lit = lit_from_tokens(quote::quote!(42.5));
        let result = lit_to_u32(&lit).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    #[should_panic(expected = "numeric attribute value is always Float or Int")]
    fn test_lit_to_u32_unreachable() {
        let lit = lit_from_tokens(quote::quote!("hello"));
        let _ = lit_to_u32(&lit);
    }
}

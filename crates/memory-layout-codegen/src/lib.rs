use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{
  parse::Parse, parse_macro_input, spanned::Spanned, Attribute, Data, DataStruct, DeriveInput,
  Error as SynError, Field, LitInt, Result as SynResult, Type
};

struct FieldInfo {
  field:           Field,
  previous_type:   Option<Type>,
  relative_offset: usize,
  absolute_offset: usize
}

struct StructInfo {
  derived: DeriveInput,
  fields:  Vec<FieldInfo>
}

impl StructInfo {
  fn get_data_struct(input: &DeriveInput) -> SynResult<&DataStruct> {
    match &input.data {
      Data::Struct(data) => Ok(data),
      Data::Enum(data) => {
        Err(SynError::new_spanned(
          data.enum_token,
          "Expected struct but found enum."
        ))
      }
      Data::Union(data) => {
        Err(SynError::new_spanned(
          data.union_token,
          "Expected struct but found union."
        ))
      }
    }
  }

  fn get_field_offset_value(attr: &Attribute) -> SynResult<usize> {
    attr
      .parse_args::<LitInt>()
      .and_then(|lit| lit.base10_parse::<usize>())
      .map_err(|_| SynError::new_spanned(attr, "Field offset must be an integer literal."))
  }

  fn get_fields(data: &DataStruct) -> SynResult<Vec<FieldInfo>> {
    let mut result = Vec::<FieldInfo>::new();

    let mut current_offset = 0usize;
    let mut previous_type: Option<Type> = None;
    for field in &data.fields {
      let field_offset = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("field_offset"));

      let offset = field_offset
        .ok_or_else(|| SynError::new_spanned(field, "Field is missing a field_offset."))
        .and_then(Self::get_field_offset_value)?;

      if current_offset > offset {
        return Err(SynError::new_spanned(
          field_offset,
          "Field offset can't be lower than its predecessor."
        ));
      }

      result.push(FieldInfo {
        field:           field.clone(),
        previous_type:   previous_type.clone(),
        relative_offset: offset - current_offset,
        absolute_offset: offset
      });

      previous_type = Some(field.ty.clone());
      current_offset = offset
    }

    Ok(result)
  }
}

impl Parse for StructInfo {
  fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
    let input: DeriveInput = input.parse()?;

    let data = Self::get_data_struct(&input)?;
    let fields = Self::get_fields(data)?;

    Ok(StructInfo {
      derived: input,
      fields
    })
  }
}

/// Allows for `field_offset`s to be defined in the struct.
/// All fields in the struct have to be annotated with a `field_offset` attribute, and must be defined in-order.
/// A `field_offset` attribute has to include a int literal, which indicates the offset the field should have.
///
/// The macro will also add `repr(C, packed)` to the struct it's applied to.
///
/// <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
/// <strong>Warning:</strong> The attribute has to be defined before any derive attributes.
/// </p>
///
/// # Example
/// ```rust
/// use ::memory_layout_codegen::memory_layout;
///
/// #[memory_layout(0x38)]
/// pub struct Example {
///   #[field_offset(0x00)]
///   a: i32,
///   #[field_offset(0x10)]
///   b: u64,
///   #[field_offset(0x18)]
///   c: f32
/// }

/// ```
///
/// Will expand to:
/// ```rust
/// #[repr(C, packed)]
/// pub struct Example {
///   #[doc(hidden)]
///   __pad0: [u8; 0usize],
///   a:      i32,
///   #[doc(hidden)]
///   __pad1: [u8; 16usize - ::core::mem::size_of::<i32>()],
///   b:      u64,
///   #[doc(hidden)]
///   __pad2: [u8; 8usize - ::core::mem::size_of::<u64>()],
///   c:      f32
///   __pad3: [u8; 8usize - ::core::mem::size_of::<u64>()],
/// }
/// ```
#[proc_macro_attribute]
pub fn memory_layout(attr: TokenStream, input: TokenStream) -> TokenStream {
  let struct_info = parse_macro_input!(input as StructInfo);

  let attr_value = parse_macro_input!(attr as Option<LitInt>);

  let desired_size = if let Some(lit) = attr_value {
    let Ok(r) = lit.base10_parse::<usize>() else {
      return quote_spanned!(
        lit.span() =>
        compile_error!("Adding `repr` manually is not supported.");
      )
      .into();
    };
    Some(r)
  } else {
    None
  };

  if let Some(attr) = struct_info
    .derived
    .attrs
    .iter()
    .find(|attr| attr.path().is_ident("repr"))
  {
    return quote_spanned!(
      attr.span() =>
      compile_error!("Adding `repr` manually is not supported.");
    )
    .into();
  }

  let mut fields = struct_info
    .fields
    .iter()
    .enumerate()
    .map(|(i, f)| {
      let ident = f.field.ident.as_ref().unwrap();
      let typename = &f.field.ty;
      let vis = &f.field.vis;
      let relative_offset = f.relative_offset;
      let previous_type = &f.previous_type;
      let pad_ident = syn::Ident::new(&format!("__pad{}", i), ident.span());
      let attrs = f
        .field
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("field_offset"));
      match previous_type {
        Some(ty) => {
          quote! {
            #[doc(hidden)]
            #pad_ident: [u8; #relative_offset - ::core::mem::size_of::<#ty>()],
            #(#attrs)*
            #vis #ident: #typename
          }
        }
        None => {
          quote! {
            #[doc(hidden)]
            #pad_ident: [u8; #relative_offset],
            #(#attrs)*
            #vis #ident: #typename
          }
        }
      }
    })
    .collect::<Vec<_>>();

  if let Some(size) = desired_size {
    let pad_ident = syn::Ident::new(
      &format!("__pad{}", struct_info.fields.len()),
      Span::call_site()
    );
    if let Some(last_field) = struct_info.fields.last() {
      let last_offset = last_field.absolute_offset;
      let prev_type = last_field.field.ty.clone();
      let Some(required_padding) = size.checked_sub(last_offset) else {
        return quote!(
          compile_error!("Desired struct size is lower than the highest field offset.");
        )
        .into();
      };

      fields.push(quote! {
        #[doc(hidden)]
        #pad_ident: [u8; #required_padding - ::core::mem::size_of::<#prev_type>()],
      })
    } else {
      fields.push(quote! {
        #[doc(hidden)]
        #pad_ident: [u8; #size],
      })
    }
  }

  let name = struct_info.derived.ident;
  let vis = struct_info.derived.vis;
  let attrs = struct_info.derived.attrs;
  let generics = struct_info.derived.generics;

  quote! {
    #[repr(C, packed)]
    #(#attrs)*
    #vis struct #name #generics {
      #(#fields),*
    }
  }
  .into()
}

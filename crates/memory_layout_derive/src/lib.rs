use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
  parse::Parse, parse_macro_input, spanned::Spanned, Attribute, Data, DataStruct, DeriveInput,
  Error as SynError, Field, LitInt, Result as SynResult, Type
};

struct FieldInfo {
  field:           Field,
  previous_type:   Option<Type>,
  relative_offset: usize
}

struct StructInfo {
  derived: DeriveInput,
  fields:  Vec<FieldInfo>
}

impl StructInfo {
  fn get_data_struct<'a>(input: &'a DeriveInput) -> SynResult<&'a DataStruct> {
    match &input.data {
      Data::Struct(data) => Ok(data),
      Data::Enum(data) => {
        Err(SynError::new_spanned(
          data.enum_token,
          "Expected struct but found enum"
        ))
      }
      Data::Union(data) => {
        Err(SynError::new_spanned(
          data.union_token,
          "Expected struct but found union"
        ))
      }
    }
  }

  fn get_field_offset_value(attr: &Attribute) -> SynResult<usize> {
    attr
      .parse_args::<LitInt>()
      .and_then(|lit| lit.base10_parse::<usize>())
      .map_err(|_| SynError::new_spanned(attr, "field offset must be usize"))
  }

  fn get_fields(data: &DataStruct) -> SynResult<Vec<FieldInfo>> {
    let mut result = Vec::<FieldInfo>::new();

    let mut current_offset = 0usize;
    let mut previous_type: Option<Type> = None;
    for field in &data.fields {
      let field_offset = field
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("field_offset"));

      let offset = field_offset
        .ok_or(SynError::new_spanned(
          field,
          "field is missing a field_offset"
        ))
        .and_then(|attr| Self::get_field_offset_value(attr))?;

      if current_offset > offset {
        return Err(SynError::new_spanned(
          field_offset,
          "field offset can't be lower than its predecessor"
        ));
      }

      result.push(FieldInfo {
        field:           field.clone(),
        previous_type:   previous_type.clone(),
        relative_offset: offset - current_offset
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
    let fields = Self::get_fields(&data)?;

    Ok(StructInfo {
      derived: input,
      fields
    })
  }
}

#[proc_macro_attribute]
pub fn memory_layout(_attr: TokenStream, input: TokenStream) -> TokenStream {
  let struct_info = parse_macro_input!(input as StructInfo);

  let fields = struct_info
    .fields
    .iter()
    .enumerate()
    .map(|(i, f)| {
      let ident = f.field.ident.as_ref().unwrap();
      let typename = &f.field.ty;
      let vis = &f.field.vis;
      let relative_offset = f.relative_offset;
      let previous_type = &f.previous_type;
      let pad_ident = syn::Ident::new(&format!("_pad{i}"), ident.span());
      match previous_type {
        Some(ty) => {
          quote! {
            #pad_ident: [u8; #relative_offset - ::std::mem::size_of::<#ty>()],
            #vis #ident: #typename
          }
        }
        None => {
          quote! {
            #pad_ident: [u8; #relative_offset],
            #vis #ident: #typename
          }
        }
      }
    })
    .collect::<Vec<_>>();

  let name = struct_info.derived.ident;
  quote! {
    #[repr(C, packed)]
    pub struct #name {
      #(#fields),*
    }
  }
  .into()
}

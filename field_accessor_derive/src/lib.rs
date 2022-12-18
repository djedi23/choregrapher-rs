extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{__private::Span, quote};
use syn::{self, Attribute, Field, Ident, Variant};

fn parse_serde_rename(attrs: Vec<Attribute>) -> Option<String> {
  // Parse les '#[serde(rename = "n")]' et retourne 'n' ou le nom du champ en minuscule
  let mut name = None;
  for attr in &attrs {
    if let syn::Meta::List(meta) = attr.parse_meta().unwrap() {
      if meta.path.segments.first().unwrap().ident == "serde" {
        let nestedmeta = meta.nested.first().unwrap();
        if let syn::NestedMeta::Meta(pmeta) = nestedmeta {
          if pmeta.path().get_ident().unwrap() == "rename" {
            if let syn::Meta::NameValue(namevalue) = pmeta {
              if let syn::Lit::Str(str) = &namevalue.lit {
                name = Some(str.value())
              }
            }
          }
        }
      }
    }
  }
  name
}

fn variant_name(variant: &Variant) -> String {
  match parse_serde_rename(variant.attrs.clone()) {
    Some(name) => name,
    None => variant.ident.to_string().to_lowercase(),
  }
}

fn field_name(field: &Field) -> String {
  match parse_serde_rename(field.attrs.clone()) {
    Some(name) => name,
    None => field.ident.clone().unwrap().to_string(),
  }
}

#[proc_macro_derive(FieldAccessor)]
pub fn field_accessor_derive(input: TokenStream) -> TokenStream {
  // Construct a representation of Rust code as a syntax tree
  // that we can manipulate
  let ast: syn::DeriveInput = syn::parse(input).unwrap();
  let name = &ast.ident;
  let generics = &ast.generics;
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

  let (field_list, field) = match ast.data {
    syn::Data::Struct(datastruct) => {
      let t: Vec<String> = datastruct.fields.iter().map(field_name).collect();

      (
        quote! {
            fn field_list(self) -> Vec<String> {
              vec![#(#t.into()),*]
            }
        },
        quote! {
            fn field(self) -> Vec<String> {
              vec![#(#t.into()),*]
            }
        },
      )
    }
    syn::Data::Enum(dataenum) => {
      // Derive an enum
      let field_list = dataenum.variants.iter().map(|variant| {
        let out = variant_name(variant);
        quote! {#out}
      });

      let variants = dataenum.variants.iter().map(|variant| {
        let variantfield = variant
          .fields
          .iter()
          .map(|_| Ident::new("_", Span::call_site()));
        let variantident = &variant.ident;
        let out = variant_name(variant);
        quote! {#name::#variantident( #(#variantfield),* ) => {#out.into()}}
      });

      (
        quote! {
            fn field_list(self) -> Vec<String> {
              vec![#(#field_list.into()),*]
            }
        },
        quote! {
                fn field(self) -> Vec<String> {
        vec![
            match self {
                            #(#variants),*
            }
        ]
                }
         },
      )
    }
    syn::Data::Union(_) => (quote! {}, quote! {}),
  };

  let gen = quote! {
    impl#impl_generics FieldAccessor for #name #ty_generics #where_clause {
  #field_list
  #field
    }
  };

  //  println!("{}", gen.to_string());

  gen.into()
}

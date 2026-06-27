use proc_macro::TokenStream;
use quote::quote;
use syn::{Field, Fields, Ident, ItemStruct, parse_macro_input};

/// Implements the entity trait and adds fields.
/// Only works on named structs, tuple/unit structs don't work.
/// Provides an implementation for default() too.
#[proc_macro_attribute]
pub fn entity(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_struct = parse_macro_input!(item as ItemStruct);

    let entity_field: Field = syn::parse_quote! {
        entity_fields: delve_shared::types::EntityFields
    };

    match &mut input_struct.fields {
        Fields::Named(fields) => {
            fields.named.push(entity_field);
        }
        Fields::Unnamed(_) | Fields::Unit => {
            let err = syn::Error::new_spanned(
                &input_struct,
                "entity cannot be applied to a tuple struct or unit struct",
            );

            return err.into_compile_error().into();
        }
    };

    let vis = &input_struct.vis;
    let ident = &input_struct.ident;
    let fields = &input_struct.fields;

    // can we avoid the clone?
    let field_names = fields
        .iter()
        .map(|f| f.ident.clone().unwrap())
        .collect::<Vec<Ident>>();

    quote! {
        #vis struct #ident #fields

        impl Default for #ident {
            fn default() -> Self {
                Self {
                    #(#field_names: Default::default()),*
                }
            }
        }

        impl delve_shared::traits::Entity for #ident {

            fn entity_fields(&self) -> &delve_shared::types::EntityFields {
                &self.entity_fields
            }

            fn entity_fields_mut(&mut self) -> &mut delve_shared::types::EntityFields {
                &mut self.entity_fields
            }

            fn apply_forces(&mut self) {
                self.entity_fields.velocity = self.entity_fields.velocity.scalar_multiply(delve_shared::constants::VELOCITY_DAMP);
                self.entity_fields.velocity += self.entity_fields.acceleration;
                self.entity_fields.position += self.entity_fields.velocity;
            }
        }
    }
    .into()
}

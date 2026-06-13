use proc_macro::TokenStream;
use quote::quote;
use syn::{Field, Fields, Ident, ItemStruct, parse_macro_input};

/// Implements the entity trait and adds fields.
/// Only works on named structs, tuple/unit structs don't work.
/// Provides an implementation for default()
#[proc_macro_attribute]
pub fn entity(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_struct = parse_macro_input!(item as ItemStruct);

    let entity_field: Field = syn::parse_quote! {
        _entity_fields: delve_shared::types::EntityFields
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
        #[derive(Debug)]
        #vis struct #ident #fields

        impl Default for #ident {
            fn default() -> Self {
                Self {
                    #(#field_names: Default::default()),*
                }
            }
        }

        impl delve_shared::traits::Entity for #ident {
            fn set_gravity(&mut self, multiplier: f32) {
                self._entity_fields.gravity_multiplier = multiplier;
            }

            fn set_rotation(&mut self, rotation_matrix: delve_shared::math::Mat3) {
                self._entity_fields.rotation = rotation_matrix;
            }

            fn set_position(&mut self, position: delve_shared::math::Vec3) {
                self._entity_fields.position = position;
            }

            fn set_velocity(&mut self, velocity: delve_shared::math::Vec3) {
                self._entity_fields.velocity = velocity;
            }

            fn add_acceleration(&mut self, acceleration: delve_shared::math::Vec3, decay: f32) {
                self._entity_fields.acceleration.push((acceleration, decay));
            }

            fn clear_acceleration(&mut self) {
                self._entity_fields.acceleration.clear();
            }

            fn apply_forces(&mut self) {
                // TODO: this is expensive because we heap alloc each time. would this be better with a
                // hash set?
                let mut new_accel = Vec::with_capacity(self._entity_fields.acceleration.len());

                for (accel, decay) in self._entity_fields.acceleration.iter_mut() {
                    *accel = accel.scalar_multiply(*decay);

                    self._entity_fields.velocity += *accel;
                    self._entity_fields.position += self._entity_fields.velocity;

                    // this is bad
                    if accel.square_magnitude() > 0.01 {
                        new_accel.push((accel.clone(), decay.clone()));
                    }
                }

                self._entity_fields.acceleration = new_accel;
            }
        }
    }
    .into()
}

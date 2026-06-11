use proc_macro::TokenStream;
use quote::quote;
use syn::{Field, Fields, ItemStruct, parse_macro_input};

#[proc_macro_attribute]
pub fn entity(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_struct = parse_macro_input!(item as ItemStruct);

    let rotation_matrix_field: Field = syn::parse_quote! {
        _entity_rotation: delve_shared::math::Mat3
    };
    let acceleration_field: Field = syn::parse_quote! {
        _entity_acceleration: Vec<(delve_shared::math::Vec3, f32)>
    };
    let velocity_field: Field = syn::parse_quote! {
        _entity_velocity: delve_shared::math::Vec3
    };
    let position_field: Field = syn::parse_quote! {
        _entity_position: delve_shared::math::Vec3
    };
    let gravity_mult_field: Field = syn::parse_quote! {
        _entity_gravity_multiplier: f32
    };

    match &mut input_struct.fields {
        Fields::Named(fields) => {
            fields.named.push(rotation_matrix_field);
            fields.named.push(acceleration_field);
            fields.named.push(velocity_field);
            fields.named.push(position_field);
            fields.named.push(gravity_mult_field);
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

    quote! {
        use delve_shared;

        #vis struct #ident #fields

        impl delve_shared::traits::Entity for #ident {
            // fn new() -> Self {
            //     Self
            // }

            fn set_gravity(&mut self, multiplier: f32) {
                self._entity_gravity_multiplier = multiplier;
            }

            fn set_rotation(&mut self, rotation_matrix: delve_shared::math::Mat3) {
                self._entity_rotation = rotation_matrix;
            }

            fn set_position(&mut self, position: delve_shared::math::Vec3) {
                self._entity_position = position;
            }

            fn set_velocity(&mut self, velocity: delve_shared::math::Vec3) {
                self._entity_velocity = velocity;
            }

            fn add_acceleration(&mut self, acceleration: delve_shared::math::Vec3, decay: f32) {
                self._entity_acceleration.push((acceleration, decay));
            }

            fn clear_acceleration(&mut self) {
                self._entity_acceleration.clear();
            }

            fn apply_forces(&mut self) {
                // TODO: this is expensive because we heap alloc each time. would this be better with a
                // hash set?
                let mut new_accel = Vec::with_capacity(self._entity_acceleration.len());

                for (accel, decay) in self._entity_acceleration.iter_mut() {
                    *accel = accel.scalar_multiply(*decay);

                    self._entity_velocity += *accel;
                    self._entity_position += self._entity_velocity;

                    // this is bad
                    if accel.square_magnitude() > 0.01 {
                        new_accel.push((accel.clone(), decay.clone()));
                    }
                }

                self._entity_acceleration = new_accel;
            }
        }
    }
    .into()
}

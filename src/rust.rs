use super::Options;
use rand::prelude::*;
use std::{error::Error, mem};
use syn::{parse::Parser, visit_mut::VisitMut};

pub fn alter_file(file: &mut syn::File, options: Options) -> Result<(), Box<dyn Error>> {
    let mut visitor = TypeRandomizationVisitor::new(options);
    visitor.visit_file_mut(file);
    Ok(())
}

#[derive(Clone, Debug)]
pub struct TypeRandomizationVisitor {
    options: Options,
    repr_attr_path: syn::Path,
    repr_c_attr: Vec<syn::Attribute>,
}

impl TypeRandomizationVisitor {
    pub fn new(options: Options) -> Self {
        Self {
            options,
            repr_attr_path: syn::parse_str("repr").unwrap_or_else(|e| {
                unreachable!(
                    "Couldn't parse \"repr\" as a `Path`!\nOriginal error: {}",
                    e
                )
            }),
            repr_c_attr: (syn::Attribute::parse_outer)
                .parse_str("#[repr(C)]")
                .unwrap_or_else(|e| {
                    unreachable!(
                        "Couldn't parse \"#[repr(C)]\" as an `Attribute`!\nOriginal error: {}",
                        e
                    )
                }),
        }
    }
}

impl VisitMut for TypeRandomizationVisitor {
    fn visit_item_struct_mut(&mut self, struct_item: &mut syn::ItemStruct) {
        use syn::{punctuated::Punctuated, *};
        let mut altered = false;
        let struct_name = struct_item.ident.to_string();
        let explicitly_excluded = self.options.exclude.is_match(&struct_name);
        let explicitly_included = self.options.include.is_match(&struct_name);
        let implicitly_excluded = struct_item
            .attrs
            .iter()
            .any(|attr| attr.path == self.repr_attr_path);
        drop(struct_name);
        if !explicitly_excluded && (explicitly_included || !implicitly_excluded) {
            if let Fields::Named(fields) = &mut struct_item.fields {
                let mut new_fields = Punctuated::new();
                mem::swap(&mut fields.named, &mut new_fields);
                let mut fields_vec: Vec<Field> =
                    new_fields.into_pairs().map(|p| p.into_value()).collect();
                self.options.rng.shuffle(&mut fields_vec);
                fields.named.extend(fields_vec);
                altered = true;
            }
        }
        if altered {
            struct_item.attrs.extend(self.repr_c_attr.iter().cloned());
        }
    }
}

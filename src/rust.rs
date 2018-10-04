use super::Options;
use rand::prelude::*;
use std::{error::Error, mem};
use syn::{*, visit_mut::VisitMut, punctuated::Punctuated};

pub fn alter_file(file: &mut syn::File, options: Options) -> Result<(), Box<dyn Error>> {
    let mut visitor = TypeRandomizationVisitor::new(options);
    visitor.visit_file_mut(file);
    Ok(())
}

#[derive(Clone, Debug)]
pub struct TypeRandomizationVisitor {
    options: Options,
    repr_attr_path: syn::Path,
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
        }
    }
}

impl VisitMut for TypeRandomizationVisitor {
    fn visit_item_struct_mut(&mut self, struct_item: &mut ItemStruct) {
        let struct_name = struct_item.ident.to_string();
        let explicitly_excluded = self.options.exclude.is_match(&struct_name);
        let explicitly_included = self.options.include.is_match(&struct_name);
        let implicitly_excluded = struct_item.attrs.iter().any(|attr| attr.path == self.repr_attr_path);
        if !explicitly_excluded && (explicitly_included || !implicitly_excluded) {
            match &mut struct_item.fields {
                Fields::Named(fields) => {
                    let mut new_fields = Punctuated::new();
                    mem::swap(&mut fields.named, &mut new_fields);
                    let mut fields_vec: Vec<Field> = new_fields.into_pairs().map(|p| p.into_value()).collect();
                    self.options.rng.shuffle(&mut fields_vec);
                    fields.named.extend(fields_vec);
                }
                _ => {}
            }
        }
    }
}

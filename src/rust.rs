use super::Options;
use rand::prelude::*;
use rayon::prelude::*;
use std::error::Error;

pub fn alter_file(file: &mut syn::File, options: Options) -> Result<(), Box<dyn Error>> {
    let mut visitor = TypeRandomizationVisitor::new(options);
    // TODO: call visitor
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

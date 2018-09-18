use rand::prelude::*;
use rayon::prelude::*;

pub fn alter_file(
    file: &mut syn::File,
    options: super::Options,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub struct TypeRandomizationVisitor<R: ?Sized + rand::Rng>(pub super::Options, R);

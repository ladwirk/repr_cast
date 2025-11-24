use repr_cast::repr_cast;

// This should fail because it mixes unit and tuple variants
#[repr_cast(u8)]
enum MixedVariants {
    Unit,
    Tuple(i32),
    Another,
}

fn main() {}

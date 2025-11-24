use repr_cast::repr_cast;

#[repr_cast(u8)]
enum EnumWithFields {
    Variant1(u32),
    Variant2,
}

fn main() {}

use repr_cast::repr_cast;

#[repr_cast(u8)]
enum EnumWithNamedFields {
    Variant1 { x: u32, y: u32 },
    Variant2,
}

fn main() {}

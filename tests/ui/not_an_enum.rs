use repr_cast::repr_cast;

#[repr_cast(u8)]
struct NotAnEnum {
    field: u32,
}

fn main() {}

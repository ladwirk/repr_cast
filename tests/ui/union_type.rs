use repr_cast::repr_cast;

#[repr_cast(u32)]
union MyUnion {
    x: u32,
    y: f32,
}

fn main() {}

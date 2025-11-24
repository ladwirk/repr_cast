use repr_cast::repr_cast;

#[repr_cast]
enum Status {
    Pending = 0,
    Active = 1,
}

fn main() {}

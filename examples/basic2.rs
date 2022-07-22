use rspc::Type;

#[derive(Type)]
pub struct User {
    pub id: i32,
}

fn main() {
    rspc::typegen::typescript_export::<User>();
}

// TODO:
// - Export types
//  - Tuple struct
//  - Unit struct
//  - Normal struct
// - Enums
// - Serde compatible types
// - UUID compatibility

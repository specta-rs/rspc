use normi::Object;

#[derive(Object)]
struct GoodStruct {
    #[normi(id)]
    id: String,
}

fn main() {}

/* A type that has no fields can never have an if field and hence is invalid */

#[derive(Object)]
struct Unit;

#[derive(Object)]
struct Unit2 {}

#[derive(Object)]
struct Unit3();

#[derive(Object)]
enum EmptyEnum {}

/* Unions are not supported */

#[derive(Object)]
union Union {
    f1: u32,
    f2: f32,
}

/*  Tuple structs */

#[derive(Object)]
struct TupleStruct1(i32); // Only single field

#[derive(Object)]
struct UnitRef(GoodStruct); // Doesn't have flatten attribute

#[derive(Object)]
struct TupleStruct1(i32, String, bool); // No id field

#[derive(Object)]
struct TupleStruct2(#[normi(id)] String, #[normi(refr)] i32); // i32 doesn't implement Object

#[derive(Object)]
struct TupleStruct3(#[normi(id, refr)] i32); // The id can't also be normalised

#[derive(Object)]
struct Unit(#[normi(id)] String); // Technically this type could be supported but using it would be useless.

/* Structs */

#[derive(Object)]
struct Struct {
    #[normi(id)]
    id: String,
    #[normi(refr)]
    arg: i32, // i32 doesn't implement Object
}

#[derive(Object)]
struct Struct1 {
    #[normi(id, refr)] // The id can't also be normalised
    id: String,
}

#[derive(Object)]
struct GenericStruct<T> {
    #[normi(id)]
    id: String,
    #[normi(refr)]
    arg: T,
}

pub type Demo = GenericStruct<i32>; // i32 doesn't implement Object


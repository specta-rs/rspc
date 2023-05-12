use rspc::selection;
use specta::{ts::inline, Type};

fn ts_export_ref<T: Type>(_t: &T) -> String {
    inline::<T>(&Default::default()).unwrap()
}

#[derive(Clone)]
#[allow(dead_code)]
struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub age: i32,
    pub password: String,
}

#[test]
fn test_selection_macros() {
    let user = User {
        id: 1,
        name: "Monty Beaumont".into(),
        email: "monty@otbeaumont.me".into(),
        age: 7,
        password: "password123".into(),
    };

    let s1 = selection!(user.clone(), { name, age });
    assert_eq!(s1.name, "Monty Beaumont".to_string());
    assert_eq!(s1.age, 7);
    assert_eq!(ts_export_ref(&s1), "{ name: string; age: number }");

    let users = vec![user; 3];
    let s2 = selection!(users, [{ name, age }]);
    assert_eq!(s2[0].name, "Monty Beaumont".to_string());
    assert_eq!(s2[0].age, 7);
    assert_eq!(ts_export_ref(&s2), "({ name: string; age: number })[]");
}

use rspc_core::Procedure;

#[derive(Debug)]
struct File;

fn main() {
    // /* Serialize */
    let y = Procedure::new(|_ctx, input| {
        let input: String = input.deserialize().unwrap();
        println!("GOT {}", input);
    });
    let result = y.exec_with_deserializer((), serde_json::Value::String("hello".to_string()));

    // /* Non-serialize */
    let y = Procedure::new(|_ctx, input| {
        let input: File = input.value().unwrap();
        println!("GOT {:?}", input);
    });
    let result = y.exec_with_value((), File);
}

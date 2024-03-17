use std::convert::Infallible;

use rspc2::{serde, Procedure, Router};

// TODO
use rspc2::serde::de::IntoDeserializer;

pub struct Primitive<T>(T);

// The error type here is just a placeholder, to contrain it.
impl<'de, T: IntoDeserializer<'de, serde::de::value::Error>>
    IntoDeserializer<'de, serde::de::value::Error> for Primitive<T>
{
    type Deserializer = T::Deserializer;

    fn into_deserializer(self) -> Self::Deserializer {
        self.0.into_deserializer()
    }
}

#[tokio::main]
async fn main() {
    let p = <Procedure>::new::<Infallible>()
        .error::<String>()
        .query(|ctx, input: i32| async move { Ok(()) });

    let router = <Router>::new().procedure("a", p).build().unwrap();

    let result = router
        .exec(
            "a",
            (),
            Primitive(42).into_deserializer(),
            rspc2::serde_json::value::Serializer,
        )
        // .await // TODO
        .unwrap();
    println!("{:?}", result);

    router
        .exec(
            "a",
            (),
            Primitive(42).into_deserializer(),
            &mut rspc2::serde_json::Serializer::new(std::io::stdout()),
        )
        // .await // TODO
        .unwrap();

    // let value = rspc::serde_json::json!(43);
    // let result = router
    //     .exec::<JsonValue, TokioRuntime>("a", (), value.into_deserializer())
    //     .await
    //     .unwrap();
    // println!("{:?}", result);

    // let result = router
    //     .exec::<JsonValue, TokioRuntime>(
    //         "a",
    //         (),
    //         &mut rspc::serde_json::Deserializer::new(rspc::serde_json::de::StrRead::new("44")),
    //     )
    //     .await
    //     .unwrap();
    // println!("{:?}", result);
}

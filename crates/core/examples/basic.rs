use std::pin::pin;

use futures::{stream::poll_fn, StreamExt};
use rspc_core::{Procedure, ProcedureStream, ResolverError};

#[derive(Debug)]
struct File;

fn main() {
    futures::executor::block_on(main2());
}

async fn main2() {
    // /* Serialize */
    // TODO

    // /* Serialize + Stream */
    let y = Procedure::new(|_ctx, input| {
        let input = input.deserialize::<String>();
        // println!("GOT {}", input);

        ProcedureStream::from_stream(futures::stream::iter(vec![
            input.map(|x| x.len()).map_err(Into::into),
            Ok(1),
            Ok(2),
            Ok(3),
            Err(ResolverError::new(500, "Not found", None::<std::io::Error>)),
        ]))
    });
    // let mut result = y.exec_with_deserializer((), serde_json::Value::String("hello".to_string()));
    // while let Some(value) = result.next(serde_json::value::Serializer).await {
    //     println!("{value:?}");
    // }

    let mut result = y.exec_with_deserializer((), serde_json::Value::Null);
    while let Some(value) = result.next(serde_json::value::Serializer).await {
        println!("{value:?}");
    }

    // // /* Non-serialize */
    // let y = Procedure::new(|_ctx, input| {
    //     let input: File = input.value().unwrap();
    //     println!("GOT {:?}", input);
    // });
    // let result = y.exec_with_value((), File);

    /* Async */
    // let y = Procedure::new(|_ctx, input| {
    //     let input: String = input.deserialize().unwrap();
    //     println!("GOT {}", input);
    //     async move {
    //         println!("Async");
    //     }
    // });
    // let result = y.exec_with_deserializer((), serde_json::Value::String("hello".to_string()));
    // let result: ProcedureStream = todo!();

    // let result = pin!(result);
    // let got = poll_fn(|cx| {
    //     let buf = Vec::new();
    //     result.poll_next(cx, &mut buf)
    // })
    // .collect()
    // .await;
    // println!("{:?}", got);

    // todo().await;
}

async fn todo() {
    println!("A");

    // Side-effect based serializer
    // let mut result: ProcedureStream = ProcedureStream::from_stream(futures::stream::iter(vec![
    //     Ok(1),
    //     Ok(2),
    //     Ok(3),
    //     Err(ResolverError::new(500, "Not found", None::<std::io::Error>)),
    // ]));

    // TODO: Clean this up + `Stream` adapter.
    // loop {
    //     let mut buf = Vec::new();
    //     let Some(result) = result
    //         .next(&mut serde_json::Serializer::new(&mut buf))
    //         .await
    //     else {
    //         break;
    //     };
    //     let _result: () = result.unwrap(); // TODO
    //     println!("{:?}", String::from_utf8_lossy(&buf));
    // }

    // Result based serializer
    let mut result: ProcedureStream = ProcedureStream::from_stream(futures::stream::iter(vec![
        Ok(1),
        Ok(2),
        Ok(3),
        Err(ResolverError::new(500, "Not found", None::<std::io::Error>)),
    ]));

    while let Some(value) = result.next(serde_json::value::Serializer).await {
        println!("{value:?}");
    }
}

// let got: Vec<String> = poll_fn(|cx| {
//     let mut buf = Vec::new(); // TODO: We alloc per poll, we only need to alloc per-value.
//     result
//         .poll_next(cx, &mut serde_json::Serializer::new(&mut buf))
//         .map(|x| x.map(|_| String::from_utf8_lossy(&buf).to_string()))
// })
// .collect()
// .await;
// println!("{:?}", got);

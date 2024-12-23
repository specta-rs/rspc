use std::sync::Arc;

// use serde::{Serialize, Serializer};

// TODO: What should this be called?
// TODO: Matching on headers too or just `Content-Type`?
// TODO: Error handling for decoding errors or no matching content-type
// TODO: `Content-Type` for incoming body decoding as well
// TODO: Client typesafety. Eg. does an endpoint require a specific content-type Eg. `File`?

// #[derive(Default)]
// pub struct Registry(Option<Arc<dyn Fn(&str, &mut dyn erased_serde::Serialize) -> bool>>);

// impl Registry {
//     // pub fn new() -> Self {
//     //     Self::default()
//     // }

//     // TODO: Remove
//     // pub fn todo(&self, handler: impl Fn(&str) -> ()) {
//     //     self.0.push(Arc::new(|ct, v| {}));
//     // }

//     pub fn r#match<S: Serialize>(&self, content_type: &str, value: S) {
//         // for f in &self.0 {
//         //     // f(content_type);
//         // }

//         todo!();
//     }
// }

// TODO: `Default` + `Debug`, `Clone`, etc

// fn todo() {
//     Registry::default().register(|ct, ser| {
//         if ct.starts_with("application/json") {
//             // TODO: We need to be able to configure `Content-Type` header

//             // ser.serialize() or ser.value()
//             // serde_json::to_writer(v, &ct).unwrap();
//             // true
//         } else {
//             // false
//         }
//     })
// }

// pub trait Request {
//     fn method(&self) -> &str;

//     // fn path(&self) -> &str;
//     // fn query(&self) -> &str;
//     // fn headers(&self);
//     // fn body(&self);
// }

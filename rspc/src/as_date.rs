use std::fmt;

pub struct AsDate<T>(T);

impl<T: fmt::Debug> fmt::Debug for AsDate<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

// TODO: Trait passthroughs (`Debug`, `Clone`, `Deserialize`, etc) + `Deref` & `Into` impls
// TODO: make generic over any `T: Serialize`???
// impl<Tz: TimeZone> Serialize for AsDate<chrono::DateTime<Tz>> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         // TODO: Should we require a `thread_local` to enable this impl so types are reusable???
//         // TODO: What if the rspc client wants it in a string format?
//         let mut s = serializer.serialize_struct("AsDate", 2)?;
//         s.serialize_field("~rspc~.date", &true)?;
//         s.serialize_field("~rspc~.value", &self.0)?;
//         s.end()
//     }
// }

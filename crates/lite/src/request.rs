pub struct RequestContext {}

// /// TODO
// // TODO: Maybe rename to `Request` or something else. Also move into Public API cause it might be used in middleware
// #[derive(Debug, Clone)]
// pub struct RequestContext {
//     pub id: u32,
//     pub kind: ProcedureKind,
//     pub path: Cow<'static, str>,
//     #[cfg(feature = "tracing")]
//     span: Option<Option<tracing::Span>>,
//     // Prevents downstream user constructing type
//     _priv: (),
// }

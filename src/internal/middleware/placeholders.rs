// mod private {
//     use std::marker::PhantomData;

//     #[doc(hidden)]
//     pub struct MissingResolver<TLayerCtx>(PhantomData<TLayerCtx>);

//     impl<TLayerCtx> MissingResolver<TLayerCtx> {
//         pub(crate) const fn new() -> Self {
//             Self(PhantomData)
//         }
//     }
// }

// pub(crate) use private::MissingResolver;

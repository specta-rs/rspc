use super::BaseProcedure;

use async_stream::stream;

// TODO: Implement this example & document how it works using shared state on `TCtx`

pub fn mount() -> rspc::Router {
    rspc::Router::builder()
        .procedure("send", {
            <BaseProcedure>::builder().query(|_, _: ()| async { Ok(env!("CARGO_PKG_VERSION")) })
        })
        .procedure("subscribe", {
            <BaseProcedure>::builder().subscription(|_, _: ()| async {
                Ok(stream! {
                    for i in 0..3 {
                        yield Ok(i);
                    }
                })
            })
        })
}

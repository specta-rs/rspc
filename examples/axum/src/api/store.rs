use rspc_openapi::OpenAPI;

use super::{BaseProcedure, Router};

pub fn mount() -> Router {
    Router::new()
        .procedure("get", {
            <BaseProcedure>::builder()
                .with(OpenAPI::get("/api/get").build())
                .query(|ctx, _: ()| async move {
                    // TODO

                    Ok("Hello From rspc!")
                })
        })
        .procedure("set", {
            <BaseProcedure>::builder()
                .with(OpenAPI::post("/api/set").build())
                .mutation(|ctx, value: String| async move {
                    // TODO

                    Ok(())
                })
        })
}

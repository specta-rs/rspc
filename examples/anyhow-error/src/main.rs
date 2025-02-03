use rspc::{Procedure, Router};

////////////////////////////////////////////////////////////////////////////////////////////////////

fn main() {
    let (_procedures, _types) = Router::new()
        .procedure(
            //
            "anyhow",
            Procedure::builder().query(anyhow_procedure),
        )
        .build()
        .expect("router should be built");
}

// Some procedure that needs `anyhow::Error` to be compatible with `rspc`.
async fn anyhow_procedure(_ctx: (), _input: ()) -> Result<String, AnyhowError> {
    let response = fallible()?; // `?` converts `anyhow::Error` into `AnyhowError`.
    Ok(response)
}

fn fallible() -> Result<String, anyhow::Error> {
    anyhow::bail!("oh no!")
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// Make `anyhow::Error` work where `std::error::Error + Send + 'static` is expected.
// NB: Define this only once; afterwards, you can import and use it anywhere.
// See: https://github.com/dtolnay/anyhow/issues/153#issuecomment-833718851
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
struct AnyhowError(#[from] anyhow::Error);

impl rspc::Error for AnyhowError {
    fn into_procedure_error(self) -> rspc::ProcedureError {
        let message = format!("something bad happened: {}", self);
        rspc::ResolverError::new(message, Some(self)).into()
    }
}

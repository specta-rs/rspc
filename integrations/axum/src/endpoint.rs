use std::error;

use rspc::{
    internal::{Serialize, Type},
    Router,
};

pub fn endpoint<TCtx, TErr: error::Error + Serialize + Type>(router: Router<TCtx, TErr>) {
    todo!();
}

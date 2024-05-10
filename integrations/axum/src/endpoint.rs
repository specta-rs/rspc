use rspc::{
    internal::{Serialize, Type},
    Router,
};

pub fn endpoint<TCtx, TErr: Serialize + Type>(router: Router<TCtx, TErr>) {
    todo!();
}

use rspc::Router;

fn main() {
    Router::<String>::new()
        .middleware()
        .query("a", || "0.1.0")
        .query("b", |ctx| "0.1.0")
        .query("c", |ctx, arg: ()| "Hello World")
        .query("d", |ctx, arg: String| async move {
            println!("{:?} {:?}", ctx, arg);
            "Hello World"
        });
}

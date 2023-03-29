use std::{borrow::Cow, sync::Arc};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pprof::criterion::{Output, PProfProfiler};

const I: usize = 10;

async fn benchmark_main(r: &Arc<rspc::Router>) {
    use rspc::internal::jsonrpc::*;
    use rspc::internal::*;

    for _ in 0..I {
        let mut response = None as Option<jsonrpc::Response>;
        handle_json_rpc(
            (),
            jsonrpc::Request {
                jsonrpc: None,
                id: RequestId::Null,
                inner: jsonrpc::RequestInner::Query {
                    path: "demo".to_string(), // TODO: Lifetime instead of allocate?
                    input: None,
                },
            },
            Cow::Borrowed(r),
            &mut response,
        )
        .await;
        let _result = black_box(response.unwrap());

        // println!("{:?}", result);
    }
}

async fn benchmark_0_1_3(r: &Arc<rspc_legacy::Router>) {
    use rspc_legacy::internal::jsonrpc::*;
    use rspc_legacy::internal::*;

    for _ in 0..I {
        let mut sender = Sender::Response(None);
        handle_json_rpc(
            (),
            jsonrpc::Request {
                jsonrpc: None,
                id: RequestId::Null,
                inner: jsonrpc::RequestInner::Query {
                    path: "demo".to_string(), // TODO: Lifetime instead of allocate?
                    input: None,
                },
            },
            &r,
            &mut sender,
            &mut SubscriptionMap::None,
        )
        .await;
        let _result = black_box(match sender {
            Sender::Response(Some(r)) => r,
            _ => unreachable!(),
        });

        // println!("{:?}", result);
    }
}

// Run the criterion benchmarks
fn bench(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("main-build-routers", |b| {
        const R: rspc::alpha::Rspc<()> = rspc::alpha::Rspc::new();
        b.iter(|| {
            for _ in 0..100 {
                black_box(
                    R.router()
                        .procedure("demo", R.query(|_, _: ()| async move { "Hello World!" }))
                        .compat()
                        .arced(),
                );
            }
        })
    });

    c.bench_function("0.1.3-build-routers", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(
                    <rspc_legacy::Router>::new()
                        .query("demo", |t| t(|_, _: ()| async move { "Hello World!" }))
                        .build()
                        .arced(),
                );
            }
        })
    });

    c.bench_function("main", |b| {
        const R: rspc::alpha::Rspc<()> = rspc::alpha::Rspc::new();
        let r = black_box(
            R.router()
                .procedure("demo", R.query(|_, _: ()| async move { "Hello World!" }))
                .compat()
                .arced(),
        );
        b.to_async(&rt).iter(|| benchmark_main(&r))
    });

    c.bench_function("0.1.3", |b| {
        let r = black_box(
            rspc_legacy::Router::new()
                .query("demo", |t| t(|_, _: ()| async move { "Hello World!" }))
                .build()
                .arced(),
        );
        b.to_async(&rt).iter(|| benchmark_0_1_3(&r))
    });

    c.bench_function("main-mw", |b| {
        const R: rspc::alpha::Rspc<()> = rspc::alpha::Rspc::new();
        let r = black_box(
            R.router()
                .procedure(
                    "demo",
                    R.with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .with(|mw, ctx| async move { mw.next(ctx) })
                        .query(|_, _: ()| async move { "Hello World!" }),
                )
                .compat()
                .arced(),
        );
        b.to_async(&rt).iter(|| benchmark_main(&r))
    });

    c.bench_function("0.1.3-mw", |b| {
        let r = black_box(
            <rspc_legacy::Router>::new()
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(())) }))
                .query("demo", |t| t(|_, _: ()| async move { "Hello World!" }))
                .build()
                .arced(),
        );
        b.to_async(&rt).iter(|| benchmark_0_1_3(&r))
    });

    // TODO: Benchmark with shit tonnes of procedures

    // TODO: Benchmarks merging super large routers
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(None)));
    targets = bench
}
criterion_main!(benches);

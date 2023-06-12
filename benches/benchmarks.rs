use std::{borrow::Cow, sync::Arc};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pprof::criterion::{Output, PProfProfiler};
use rspc::internal::exec::{self, Executor, ExecutorResult, NoOpSubscriptionManager, TokioRuntime};

const I: usize = 100;

async fn benchmark_main(e: &Executor<(), TokioRuntime>) {
    for _ in 0..I {
        let response = match e.execute(
            (),
            exec::Request::Query {
                path: Cow::Borrowed("demo"),
                input: None,
            },
            &mut (None as Option<NoOpSubscriptionManager>),
        ) {
            ExecutorResult::FutureResponse(fut) => fut.await,
            ExecutorResult::Response(resp) => resp,
            ExecutorResult::None => unreachable!(),
        };

        let _result = black_box(response);

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
        const R: rspc::Rspc<()> = rspc::Rspc::new();
        b.iter(|| {
            for _ in 0..100 {
                black_box(
                    R.router()
                        .procedure("demo", R.query(|_, _: ()| async move { "Hello World!" }))
                        .build()
                        .unwrap()
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
        const R: rspc::Rspc<()> = rspc::Rspc::new();
        let r = black_box(Executor::new(
            R.router()
                .procedure("demo", R.query(|_, _: ()| async move { "Hello World!" }))
                .build()
                .unwrap()
                .arced(),
        ));
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
        const R: rspc::Rspc<()> = rspc::Rspc::new();
        let r = black_box(Executor::new(
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
                .build()
                .unwrap()
                .arced(),
        ));
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

    c.bench_function("main-53-procedures", |b| {
        const R: rspc::Rspc<()> = rspc::Rspc::new();
        let r = black_box(Executor::new(
            R.router()
                .procedure("a", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("b", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("c", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("d", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("e", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("f", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("g", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("h", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("i", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("j", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("k", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("l", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("m", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("n", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("o", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("p", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("q", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("r", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("s", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("t", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("u", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("v", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("w", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("x", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("y", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("z", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("aa", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("ab", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("ac", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("ad", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("ae", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("af", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("ag", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("ah", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("ai", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("aj", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("ak", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("al", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("am", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("an", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("ao", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("ap", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("aq", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("ar", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("as", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("at", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("au", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("av", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("aw", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("ax", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("ay", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("az", R.query(|_, _: ()| async move { "Hello World!" }))
                .procedure("demo", R.query(|_, _: ()| async move { "Hello World!" }))
                .build()
                .unwrap()
                .arced(),
        ));
        b.to_async(&rt).iter(|| benchmark_main(&r))
    });

    c.bench_function("0.1.3-53-procedures", |b| {
        let r = black_box(
            <rspc_legacy::Router>::new()
                .query("a", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("b", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("c", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("d", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("e", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("f", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("g", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("h", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("i", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("j", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("k", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("l", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("m", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("n", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("o", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("p", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("q", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("r", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("s", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("t", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("u", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("v", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("w", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("x", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("y", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("z", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("aa", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("ab", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("ac", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("ad", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("ae", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("af", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("ag", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("ah", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("ai", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("aj", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("ak", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("al", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("am", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("an", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("ao", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("ap", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("aq", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("ar", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("as", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("at", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("au", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("av", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("aw", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("ax", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("ay", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("az", |t| t(|_, _: ()| async move { "Hello World!" }))
                .query("demo", |t| t(|_, _: ()| async move { "Hello World!" }))
                .build()
                .arced(),
        );
        b.to_async(&rt).iter(|| benchmark_0_1_3(&r))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(None)));
    targets = bench
}
criterion_main!(benches);

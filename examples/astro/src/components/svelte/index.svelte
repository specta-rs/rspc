<script lang="ts">
    import { initRspc, httpLink, wsLink } from "@rspc/client";
    import { tauriLink } from "@rspc/tauri";
    import { QueryClient, QueryClientProvider } from "@tanstack/svelte-query";

    // Export from Rust. Run `cargo run -p example-axum` to start server and export it!
    import type { Procedures } from "../../bindings.ts";

    import Example from "./Example.svelte";

    const fetchQueryClient = new QueryClient();
    const fetchClient = initRspc<Procedures>({
        links: [
            // loggerLink(),

            httpLink({
                url: "http://localhost:4000/rspc",

                // You can enable batching -> This is generally a good idea unless your doing HTTP caching
                // batch: true,

                // You can override the fetch function if required
                // fetch: (input, init) => fetch(input, { ...init, credentials: "include" }), // Include Cookies for cross-origin requests

                // Provide static custom headers
                // headers: {
                //   "x-demo": "abc",
                // },

                // Provide dynamic custom headers
                // headers: ({ op }) => ({
                //   "x-procedure-path": op.path,
                // }),
            }),
        ],
    });

    const wsQueryClient = new QueryClient();
    const wsClient = initRspc<Procedures>({
        links: [
            // loggerLink(),

            wsLink({
                url: "ws://localhost:4000/rspc/ws",
            }),
        ],
    });

    const tauriQueryClient = new QueryClient();
    const tauriClient = initRspc<Procedures>({
        links: [
            // loggerLink(),

            tauriLink(),
        ],
    });
</script>

<div style="backgroundColor: 'rgba(50, 205, 50, .5)'">
    <h1>Svelte</h1>
    <rspc.Provider client={fetchClient} queryClient={fetchQueryClient}>
        <QueryClientProvider client={fetchQueryClient}>
            <Example name="Fetch Transport" />
        </QueryClientProvider>
    </rspc.Provider>

    <rspc.Provider client={wsClient} queryClient={wsQueryClient}>
        <QueryClientProvider client={wsQueryClient}>
            <Example name="Websocket Transport" />
        </QueryClientProvider>
    </rspc.Provider>

    <rspc.Provider client={tauriClient} queryClient={tauriQueryClient}>
        <QueryClientProvider client={tauriQueryClient}>
            <Example name="Tauri Transport" />
        </QueryClientProvider>
    </rspc.Provider>
</div>

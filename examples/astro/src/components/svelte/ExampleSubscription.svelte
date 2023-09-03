<script lang="ts">
    import { createSvelteQueryHooks } from "@rspc/svelte-query";
    import type { Procedures } from "../../bindings.ts";

    const rspc = createSvelteQueryHooks<Procedures>();

    export let rerenderProp: string;
    let i = 0;

    rspc.createSubscription(["pings"], {
        onData(_) {
            i = i + 1;
        },
    });
</script>

<p>
    Pings received: {i}
    {rerenderProp}
</p>

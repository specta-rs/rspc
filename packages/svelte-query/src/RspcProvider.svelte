<script lang="ts">
    import { Client } from "@rspc/client";
    import { QueryClient } from "@tanstack/svelte-query";
    import { setRspcClientContext } from "./context";

    export let client: Client<any>;
    export let queryClient: QueryClient;

    setRspcClientContext({ client, queryClient });
</script>

<slot />

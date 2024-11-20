<script lang="ts">
	import type { Client } from "@rspc/client";
	import type { QueryClient } from "@tanstack/svelte-query";
	import { setRspcClientContext } from "./context";

	export let client: Client<any>;
	export let queryClient: QueryClient;

	setRspcClientContext({ client, queryClient });
</script>

<slot />

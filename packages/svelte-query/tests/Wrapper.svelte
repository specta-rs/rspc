<script lang="ts">
import { QueryClientProvider } from "@tanstack/svelte-query";
import { RspcProvider } from "../src";
import { client, queryClient } from "./utils";
</script>

<RspcProvider {client} {queryClient}>
	<QueryClientProvider>
		<slot />
	</QueryClientProvider>
</RspcProvider>

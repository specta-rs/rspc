<script lang="ts">
  import { onDestroy, onMount } from 'svelte'
  import { AlphaClient } from "@rspc/client"
  import { QueryClient } from "@tanstack/svelte-query"
  import { setRspcClientContext } from './context'

  export let client: AlphaClient<any>;
  export let queryClient: QueryClient;

  setRspcClientContext({client, queryClient})
</script>

<QueryClientProvider client={queryClient}>
	<slot />
</QueryClientProvider>

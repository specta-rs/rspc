<script lang="ts">
	import { createClient } from "@rspc/client";
	import { createSvelteQueryProxy } from "@rspc/svelte-query";
	import type { Procedures } from "../../../axum/bindings";

	const client = createClient<Procedures>();

	client.store.get.query();

	const rspc = createSvelteQueryProxy<Procedures>();

	rspc.store.get.createQuery();
	rspc.store.set.createMutation();
	rspc.chat.subscribe.createSubscription();
</script>

<script lang="ts">
    import { createSvelteQueryHooks } from "@rspc/svelte-query";
    import type { Procedures } from "../../bindings.ts";

    import ExampleSubscription from "./ExampleSubscription.svelte";

    export let name: string;

    const rspc = createSvelteQueryHooks<Procedures>();

    let rerenderProp = Date.now().toString();
    const version = rspc.createQuery(["version"]);
    const transformMe = rspc.createQuery(["transformMe"]);
    const echo = rspc.createQuery(["echo", "Hello From Frontend!"]);
    const sendMsg = rspc.createMutation("sendMsg");
    const error = rspc.createQuery(["error"], {
        retry: false,
    });

    let subId: number | null = null;
    let enabled = true;

    rspc.createSubscription(["testSubscriptionShutdown"], {
        enabled,
        onData(msg) {
            subId = msg;
        },
    });
</script>

<div style="border: 'black 1px solid'">
    <h1>{name}</h1>
    <p>Using rspc version: {$version.data}</p>
    <p>Echo response: {$echo.data}</p>
    <p>Error returned: {JSON.stringify($error.error)}</p>
    <p>Transformed Query: {$transformMe.data}</p>
    <ExampleSubscription {rerenderProp} />
    <button on:click={() => (rerenderProp = Date.now().toString())}>
        Rerender subscription
    </button>
    <button
        on:click={() => $sendMsg.mutate("Hello!")}
        disabled={$sendMsg.isLoading}
    >
        Send Msg!
    </button>
    <br />
    <input
        type="checkbox"
        on:click={(e) => (enabled = e.currentTarget.checked)}
        value="false"
        disabled={subId === null}
    />
    {`${enabled ? "Enabled" : "Disabled"} ${subId}`}
</div>

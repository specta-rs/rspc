import { SvelteComponentTyped } from "svelte";
import { AlphaClient } from "@rspc/client";
import { QueryClient } from "@tanstack/svelte-query";
declare const __propDef: {
    props: {
        client: AlphaClient<any>;
        queryClient: QueryClient;
    };
    events: {
        [evt: string]: CustomEvent<any>;
    };
    slots: {
        default: {};
    };
};
export type RspcProviderProps = typeof __propDef.props;
export type RspcProviderEvents = typeof __propDef.events;
export type RspcProviderSlots = typeof __propDef.slots;
export default class RspcProvider extends SvelteComponentTyped<RspcProviderProps, RspcProviderEvents, RspcProviderSlots> {
}
export {};
//# sourceMappingURL=RspcProvider.svelte.d.ts.map
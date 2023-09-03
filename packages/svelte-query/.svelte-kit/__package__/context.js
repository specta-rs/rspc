import { getContext, setContext } from "svelte";
const _contextKey = "$$_rspcClient";
/** Retrieves a Client from Svelte's context */
export const getRspcClientContext = () => {
    const ctx = getContext(_contextKey);
    if (!ctx) {
        throw new Error("No rspc Client was found in Svelte context. Did you forget to wrap your component with RspcProvider?");
    }
    return ctx;
};
/** Sets a Client on Svelte's context */
export const setRspcClientContext = (client) => {
    setContext(_contextKey, client);
};

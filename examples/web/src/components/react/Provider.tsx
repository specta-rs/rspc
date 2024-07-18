import { QueryClientProvider } from "@tanstack/react-query";
import type { PropsWithChildren } from "react";

import { client, queryClient, rspc } from ".";

export function Provider({ children }: PropsWithChildren) {
	return (
		<rspc.Provider client={client} queryClient={queryClient}>
			<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
		</rspc.Provider>
	);
}

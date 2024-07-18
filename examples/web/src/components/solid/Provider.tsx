/** @jsxImportSource solid-js */

import { QueryClientProvider } from "@tanstack/solid-query";
import { ParentProps } from "solid-js";

import { client, queryClient, rspc } from ".";

export function Provider(props: ParentProps) {
	return (
		<rspc.Provider client={client} queryClient={queryClient}>
			<QueryClientProvider client={queryClient}>
				{props.children}
			</QueryClientProvider>
		</rspc.Provider>
	);
}

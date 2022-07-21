import {
  createClient,
  createReactQueryHooks,
  FetchTransport,
  WebsocketTransport,
} from "@rspc/client";
import { QueryClient, UseQueryOptions } from "@tanstack/react-query";
import React from "react";
import ReactDOM from "react-dom/client";
import type { Operations } from "../ts/index"; // You must run the axum example for these bindings to generate
import type { LibraryArgs } from "../ts/LibraryArgs";

export const queryClient = new QueryClient();

export const rspc = createReactQueryHooks<Operations>();

type LibraryQueries = Extract<Operations["queries"], { arg: LibraryArgs }>;

type LibraryArguments<K extends LibraryQueries["key"]> = Omit<
  Extract<LibraryQueries, { key: "libraryThingsWithArg" }>["arg"],
  keyof LibraryArgs
>;

function useLibraryQuery<K extends LibraryQueries["key"]>(
  key: K,
  args?: LibraryArguments<K>,
  options?: UseQueryOptions<
    Extract<Operations["queries"], { key: K }>["result"]
  >
) {
  const library_id = "todo";

  return rspc.useQuery(
    [library_id, key],
    {
      library_id,
      ...args,
    },
    options
  );
}

const client = createClient<Operations>({
  transport: new WebsocketTransport("ws://localhost:4000/rspcws"),
  // transport: new FetchTransport("http://localhost:4000/rspc"),
});

function App() {
  // useLibraryQuery("version"); // INVALID
  useLibraryQuery("libraryThings", undefined); // VALID
  useLibraryQuery("libraryThingsWithArg", {
    demo: "Hello World",
  }); // VALID

  // const { data: version } = rspc.useQuery("version");
  // const { data: library_id } = useQueryWithCtx("version");
  // const { data } = rspc.useQuery("libraryThings", {
  //   library_id: "todo",
  // });

  return (
    <>
      {/* <h1>Using rspc version: {version}</h1> */}
      {/* <p>In library: {library_id}</p> */}
    </>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <rspc.Provider client={client} queryClient={queryClient}>
      <App />
    </rspc.Provider>
  </React.StrictMode>
);

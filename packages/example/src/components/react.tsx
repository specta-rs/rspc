import {
  ClientTransformer,
  createClient,
  FetchTransport,
  OperationKey,
  OperationType,
  WebsocketTransport,
} from "@rspc/client";
import { createReactQueryHooks } from "@rspc/react";
import { QueryClient } from "@tanstack/react-query";
import React, { useState } from "react";

import type { Operations } from "../../bindings";

export const rspc = createReactQueryHooks<Operations>();

const myCustomTransformer: ClientTransformer = {
  serialize(type: OperationType, key: OperationKey) {
    return key;
  },
  deserialize(type: OperationType, key: OperationKey, data: any) {
    if (key[0] === "transformMe") {
      data += " transformed";
    }
    return data;
  },
};

export const fetchQueryClient = new QueryClient();
const fetchClient = createClient<Operations>({
  transport: new FetchTransport("http://localhost:4000/rspc"),
  transformer: myCustomTransformer,
});

export const wsQueryClient = new QueryClient();
const wsClient = createClient<Operations>({
  transport: new WebsocketTransport("ws://localhost:4000/rspcws"),
  transformer: myCustomTransformer,
});

function Example({ name }: { name: string }) {
  const [rerenderProp, setRendererProp] = useState(Date.now().toString());
  const { data: version } = rspc.useQuery(["version"]);
  const { data: transformMe } = rspc.useQuery(["transformMe"]);
  const { data: echo } = rspc.useQuery(["echo", "Hello From Frontend!"]);
  const { mutate, isLoading } = rspc.useMutation("sendMsg");
  const { error } = rspc.useQuery(["error"], {
    retry: false,
  });

  return (
    <div
      style={{
        border: "black 1px solid",
      }}
    >
      <h1>{name}</h1>
      <p>Using rspc version: {version}</p>
      <p>Echo response: {echo}</p>
      <p>
        Error returned: {error?.code} {error?.message}
      </p>
      <p>Transformed Query: {transformMe}</p>
      <ExampleSubscription rerenderProp={rerenderProp} />
      <button onClick={() => setRendererProp(Date.now().toString())}>
        Rerender subscription
      </button>
      <button onClick={() => mutate("Hello!")} disabled={isLoading}>
        Send Msg!
      </button>
    </div>
  );
}

function ExampleSubscription({ rerenderProp }: { rerenderProp: string }) {
  const [i, setI] = useState(0);
  rspc.useSubscription(["pings"], {
    onNext(msg) {
      setI((i) => i + 1);
    },
  });

  return (
    <p>
      Pings received: {i} {rerenderProp}
    </p>
  );
}

export default function App() {
  return (
    <React.StrictMode>
      <div
        style={{
          backgroundColor: "rgba(50, 205, 50, .5)",
        }}
      >
        <h1>React</h1>
        <rspc.Provider client={fetchClient} queryClient={fetchQueryClient}>
          <Example name="Fetch Transport" />
        </rspc.Provider>
        <rspc.Provider client={wsClient} queryClient={wsQueryClient}>
          <Example name="Websocket Transport" />
        </rspc.Provider>
      </div>
    </React.StrictMode>
  );
}

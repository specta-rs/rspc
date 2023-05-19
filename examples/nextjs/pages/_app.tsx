import "../styles/globals.css";
import type { AppProps } from "next/app";
import { client, queryClient, RSPCProvider } from "../src/rspc";

const client_todo = client as any; // TODO: Fix this

function MyApp({ Component, pageProps }: AppProps) {
  return (
    <RSPCProvider client={client_todo} queryClient={queryClient}>
      <Component {...pageProps} />
    </RSPCProvider>
  );
}

export default MyApp;

import "../styles/globals.css";
import type { AppProps } from "next/app";
import { client, queryClient, RSPCProvider } from "../src/rspc";

function MyApp({ Component, pageProps }: AppProps) {
  return (
    <RSPCProvider client={client as any} queryClient={queryClient}>
      <Component {...pageProps} />
    </RSPCProvider>
  );
}

export default MyApp;

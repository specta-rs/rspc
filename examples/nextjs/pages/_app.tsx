import "../styles/globals.css";
import type { AppProps } from "next/app";
import { client, rspc } from "../src/rspc";
import { QueryClient } from "@tanstack/react-query";

const queryClient = new QueryClient();

function MyApp({ Component, pageProps }: AppProps) {
  return (
    <rspc.Provider client={client} queryClient={queryClient}>
      <Component {...pageProps} />
    </rspc.Provider>
  );
}

export default MyApp;

import "../styles/globals.css";
import type { AppProps } from "next/app";
import { client, queryClient, rspc } from "../src/rspc";

function MyApp({ Component, pageProps }: AppProps) {
  return (
    <rspc.Provider client={client} queryClient={queryClient}>
      <Component {...pageProps} />
    </rspc.Provider>
  );
}

export default MyApp;

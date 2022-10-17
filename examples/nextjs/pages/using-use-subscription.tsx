import { NextPage } from "next";
import Head from "next/head";
import { useState } from "react";
import { useSubscription } from "../src/rspc";
import styles from "../styles/Home.module.css";

const UsingUseSubscription: NextPage = () => {
  const [pings, setPings] = useState(0);

  useSubscription(["subscriptions.pings"], {
    onData: () => setPings((currentPings) => currentPings + 1),
  });

  return (
    <div className={styles.container}>
      <Head>
        <title>Using useSubscription | RSPC Example with Next.js</title>
      </Head>

      <main className={styles.main}>
        <h1 className={styles.title}>
          <code>useSubscription</code>
        </h1>

        <p className={styles.description}>WS Pings received: {pings}</p>
      </main>
    </div>
  );
};

export default UsingUseSubscription;

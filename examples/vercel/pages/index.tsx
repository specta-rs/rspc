import type { NextPage } from "next";
import Head from "next/head";
import Link from "next/link";
import styles from "../styles/Home.module.css";

const Home: NextPage = () => (
  <div className={styles.container}>
    <Head>
      <title>RSPC Example with Next.js</title>
    </Head>

    <main className={styles.main}>
      <h1 className={styles.title}>
        Welcome to{" "}
        <a href="https://rspc.otbeaumont.me" target="_blank">
          RSPC
        </a>{" "}
        with{" "}
        <a href="https://nextjs.org" target="_blank">
          Next.js!
        </a>
      </h1>

      <div className={styles.grid}>
        <Link href="/using-use-query" className={styles.card}>
          <h2>Using useQuery &rarr;</h2>
        </Link>

        <Link href="/using-use-mutation" className={styles.card}>
          <h2>Using useMutation &rarr;</h2>
        </Link>

        <Link href="/using-ssp" className={styles.card}>
          <h2>Using ServerSideProps &rarr;</h2>
        </Link>
      </div>
    </main>
  </div>
);

export default Home;

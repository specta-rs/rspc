import { GetServerSideProps, NextPage } from "next";
import { client } from "../src/rspc";
import Head from "next/head";
import styles from "../styles/Home.module.css";

interface UsingServerSideProps {
  data?: string;
  error?: string;
}

export const getServerSideProps: GetServerSideProps<
  UsingServerSideProps
> = async () => {
  try {
    return { props: { data: await client.query(["version"]) } };
  } catch (error) {
    return { props: { error: (error as Error)?.message } };
  }
};

const UsingServerSideProps: NextPage<UsingServerSideProps> = ({
  data,
  error,
}) => (
  <div className={styles.container}>
    <Head>
      <title>Using getServerSideProps | RSPC Example with Next.js</title>
    </Head>

    <main className={styles.main}>
      <h1 className={styles.title}>
        <code>getServerSideProps</code>
      </h1>
      <p className={styles.description}>
        {data && `The server version is: ${data}`}
        {error}
      </p>
    </main>
  </div>
);

export default UsingServerSideProps;

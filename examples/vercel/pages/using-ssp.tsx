import { GetServerSideProps, NextPage } from "next";
import { client } from "../src/rspc";
import Head from "next/head";
import styles from "../styles/Home.module.css";

type UsingServerSideProps =
  | {
      data: string;
    }
  | {
      error: string;
    };

export const getServerSideProps: GetServerSideProps<
  UsingServerSideProps
> = async () => {
  const result = await client.query(["version"]);

  const props =
    result.status === "ok"
      ? { data: result.data }
      : { error: JSON.stringify(result.error) };

  return { props };
};

const UsingServerSideProps: NextPage<UsingServerSideProps> = (props) => (
  <div className={styles.container}>
    <Head>
      <title>Using getServerSideProps | RSPC Example with Next.js</title>
    </Head>

    <main className={styles.main}>
      <h1 className={styles.title}>
        <code>getServerSideProps</code>
      </h1>
      <p className={styles.description}>
        {"data" in props &&
          props.data &&
          `The server version is: ${props.data}`}
        {"error" in props && props.error}
      </p>
    </main>
  </div>
);

export default UsingServerSideProps;

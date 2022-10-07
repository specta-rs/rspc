import type { Component } from "solid-js";
import Comp from "./Comp";
import rspc from "./query.axum";

const App: Component = () => {
  const { data } = rspc.createQuery(() => ["version"]);

  return (
    <>
      <h1>Hello world!!!! You are running v{data}</h1>
      <Comp />
    </>
  );
};

export default App;

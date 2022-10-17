import { rspc } from "./utils/rspc";

function App() {
  const { data } = rspc.createQuery(() => ["version"]);

  return <h1>Hello world!!!! You are running v{data}</h1>;
}

export default App;

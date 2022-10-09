import rspc from "./utils/rspc";

function App() {
  const { data } = rspc.useQuery(["version"]);

  return (
    <div>
      <h1>You are running v{data}</h1>
      <h3>This data is from the rust side</h3>
    </div>
  );
}

export default App;

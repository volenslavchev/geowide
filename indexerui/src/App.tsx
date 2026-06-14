import { useApi } from "./api";
import { DataViewer } from "./components/DataViewer";
import { Home } from "./components/Home";

function App() {
  const { fetch, data, isLoading } = useApi();

  return (
    <main>
      {data ? (
        <DataViewer data={data} />
      ) : (
        <Home onSearch={(file) => fetch(file)} isLoading={isLoading} />
      )}
    </main>
  );
}

export default App;

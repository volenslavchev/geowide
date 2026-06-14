import { useState } from "react";
import type { IndexerResponse } from "../../api";
import { Map } from "./Map";
import { Sidebar } from "./Sidebar";

type DataViewerProps = {
  data: IndexerResponse;
};

export const DataViewer: React.FC<DataViewerProps> = ({ data }) => {
  const [activeLoc, setActiveLoc] = useState(0);

  return (
    <div className="data-viewer-layout">
      <Sidebar
        data={data.locations}
        activeLocation={activeLoc}
        onLocationClick={(_loc, ind) => setActiveLoc(ind)}
      />
      <Map result={data.locations[activeLoc]} />
    </div>
  );
};

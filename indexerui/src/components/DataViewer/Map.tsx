import { MapContainer, Marker, TileLayer } from "react-leaflet";
import type { IndexResult } from "../../api";
import { useEffect, useRef } from "react";
import { useMap } from "react-leaflet";

import "leaflet/dist/leaflet.css";
import { Map as LeafletMap } from "leaflet";

type MapProps = {
  result?: IndexResult;
};

export const Map: React.FC<MapProps> = ({ result }) => {
  const mapRef = useRef<LeafletMap | null>(null);

  useEffect(() => {
    if (!result) return;
    mapRef.current?.flyTo([result.lat, result.lon], 18, {
      duration: 0.5,
    });
  }, [result]);

  if (!result) return <p>No results to show.</p>;

  return (
    <div className="map">
      <MapContainer
        center={[result.lat, result.lon]}
        zoom={12}
        preferCanvas={true}
        ref={mapRef}
      >
        <MapAutoResizer />
        <TileLayer
          attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>'
          url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
          className="map-tiles"
        />
        <Marker position={[result.lat, result.lon]}></Marker>
      </MapContainer>
    </div>
  );
};

const MapAutoResizer = () => {
  const map = useMap();

  useEffect(() => {
    const resizeObserver = new ResizeObserver(() => {
      map.invalidateSize();
    });

    const container = map.getContainer();
    resizeObserver.observe(container);

    return () => resizeObserver.disconnect();
  }, [map]);

  return null;
};

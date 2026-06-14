import type { IndexResult } from "../../api";

type SidebarProps = {
  data: IndexResult[];
  activeLocation: number;
  onLocationClick(location: IndexResult, ind: number): void;
};

export const Sidebar: React.FC<SidebarProps> = ({
  data,
  onLocationClick,
  activeLocation,
}) => {
  const item = data[activeLocation];
  return (
    <div className="sidebar">
      <div className="location-list">
        {data.map((loc, ind) => (
          <div
            className={`location${activeLocation === ind ? " active" : ""}`}
            onClick={() => onLocationClick(loc, ind)}
          >
            Location {ind + 1} - {loc.score.toFixed(3)}
          </div>
        ))}
      </div>
      <div className="google-maps">
        <iframe
          src={`https://www.google.com/maps/embed/v1/streetview?key=AIzaSyDBAuSHlsFkZVz0fcvucDD6G0C3xwveEvI&location=${item.lat},${item.lon}&heading=210&pitch=10&fov=35`}
        ></iframe>
      </div>
    </div>
  );
};

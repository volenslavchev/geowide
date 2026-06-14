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
            <span>
              Location {ind + 1} - {loc.score.toFixed(3)}
            </span>
            <a
              href={`https://www.google.com/maps/@?api=1&map_action=pano&viewpoint=${loc.lat},${loc.lon}`}
              target="_blank"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="16"
                height="16"
                fill="currentColor"
                viewBox="0 0 16 16"
              >
                <path
                  fill-rule="evenodd"
                  d="M8.636 3.5a.5.5 0 0 0-.5-.5H1.5A1.5 1.5 0 0 0 0 4.5v10A1.5 1.5 0 0 0 1.5 16h10a1.5 1.5 0 0 0 1.5-1.5V7.864a.5.5 0 0 0-1 0V14.5a.5.5 0 0 1-.5.5h-10a.5.5 0 0 1-.5-.5v-10a.5.5 0 0 1 .5-.5h6.636a.5.5 0 0 0 .5-.5"
                />
                <path
                  fill-rule="evenodd"
                  d="M16 .5a.5.5 0 0 0-.5-.5h-5a.5.5 0 0 0 0 1h3.793L6.146 9.146a.5.5 0 1 0 .708.708L15 1.707V5.5a.5.5 0 0 0 1 0z"
                />
              </svg>
            </a>
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

import { useState, type ChangeEvent } from "react";

type HomeProps = {
  onSearch(file: File): void;
  isLoading?: boolean;
};

export const Home: React.FC<HomeProps> = ({ onSearch, isLoading }) => {
  const [selectedFile, setSelectedFile] = useState<File>();
  const [previewURL, setPreviewURL] = useState<string>();

  const handleFileChange = (event: ChangeEvent<HTMLInputElement>) => {
    const fileList = event.target.files;
    if (fileList && fileList.length > 0) {
      setSelectedFile(fileList[0]);
      const objectUrl = URL.createObjectURL(fileList[0]);
      setPreviewURL(objectUrl);
    }
  };

  return (
    <div className="home-layout">
      <div className="loading-bar">
        {isLoading && (
          <svg viewBox="0 0 100 100">
            <circle
              fill="none"
              stroke="currentColor"
              strokeWidth="4"
              strokeMiterlimit="10"
              cx="50"
              cy="50"
              r="48"
            />
            <line
              fill="none"
              strokeLinecap="round"
              stroke="currentColor"
              strokeWidth="4"
              strokeMiterlimit="10"
              x1="50"
              y1="50"
              x2="85"
              y2="50.5"
            >
              <animateTransform
                attributeName="transform"
                dur="2s"
                type="rotate"
                from="0 50 50"
                to="360 50 50"
                repeatCount="indefinite"
              />
            </line>
            <line
              fill="none"
              strokeLinecap="round"
              stroke="currentColor"
              strokeWidth="4"
              strokeMiterlimit="10"
              x1="50"
              y1="50"
              x2="49.5"
              y2="74"
            >
              <animateTransform
                attributeName="transform"
                dur="15s"
                type="rotate"
                from="0 50 50"
                to="360 50 50"
                repeatCount="indefinite"
              />
            </line>
          </svg>
        )}
      </div>
      <div className="img-selector">
        {selectedFile ? (
          <>
            <div
              className="image"
              style={{ backgroundImage: `url("${previewURL}")` }}
            ></div>
            <p>{selectedFile.name}</p>
          </>
        ) : (
          <p>No image uploaded</p>
        )}
        <input type="file" onChange={handleFileChange} />
        <div className={`button${selectedFile ? "" : " primary"}`}>
          <input
            type="file"
            id="file-upload"
            onChange={handleFileChange}
            accept="image/*"
          />
          <label htmlFor="file-upload">Upload</label>
        </div>
      </div>
      <p className="subtitle">
        Or view the indexed area{" "}
        <a href="/varna.html" target="_blank">
          here
        </a>
      </p>
      {selectedFile && (
        <button className="primary" onClick={() => onSearch(selectedFile)}>
          Search
        </button>
      )}
    </div>
  );
};

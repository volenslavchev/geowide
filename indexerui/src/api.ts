import { useState } from "react";
import Compressor from "compressorjs";

export type IndexResult = {
  lon: number;
  lat: number;
  score: number;
};

export type IndexerResponse = {
  time: number;
  locations: IndexResult[];
};

export function useApi() {
  const [response, setResponse] = useState<IndexerResponse>();
  const [isLoading, setIsLoading] = useState<boolean>();
  const [isError, setIsError] = useState<boolean>();

  const makeApiCall = async (imageBytes: Blob) => {
    setResponse(undefined);
    setIsLoading(true);
    new Compressor(imageBytes, {
      quality: 0.75,
      mimeType: "image/webp",
      async success(result) {
        try {
          const response = await fetch(import.meta.env.VITE_BACKEND_URL, {
            method: "POST",
            headers: {
              "Content-Type": "application/octet-stream",
            },
            body: result,
          });
          if (!response.ok) {
            setIsLoading(false);
            setIsError(true);
            return;
          }
          const json = (await response.json()) as IndexerResponse;
          setResponse(json);
          setIsLoading(false);
        } catch {
          setIsLoading(false);
          setIsError(true);
        }
      },
      error(err) {
        console.error(err.message);
      },
    });
  };

  return {
    data: response,
    isLoading,
    isError,
    fetch: makeApiCall,
  };
}

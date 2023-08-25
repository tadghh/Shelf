/* eslint-disable camelcase */
import { invoke, convertFileSrc } from "@tauri-apps/api/tauri";
import { useEffect, useState } from "react";
import BookCover from "./book-cover";
import { isValidDirectoryPath } from "@/lib/regex";
import NoDirectory from "../shelf/no-directory";

export default function BookDashboard() {
  const [imageData, setImageData] = useState([]);
  const [titleData, setTitleData] = useState([]);
  const [directoryStatus, setDirectoryStatus] = useState(false);
  const [imagesStatus, setImagesStatus] = useState();
  const updateTitleAndImageData = (titles, images) => {
    setTitleData(titles);
    setImageData(images);
  };

  useEffect(() => {
    async function loadImages() {
      const start = performance.now();
      console.log("yo");
      const bookCovers = await invoke("initialize_books");
      const bookCoverPaths = await Promise.all(
        bookCovers.map(async (book) => {
          return convertFileSrc(book.cover_location);
        })
      );

      updateTitleAndImageData(bookCovers, bookCoverPaths);
      const end = performance.now();
      const executionTime = end - start;

      console.log(`Execution time: ${executionTime} milliseconds`);
      setImagesStatus(true);
    }

    invoke("get_configuration_option", {
      option_name: "book_folder_location",
    }).then((data) => {
      if (isValidDirectoryPath(data)) {
        console.log("good");
        setDirectoryStatus(data);
        loadImages();
      } else {
        console.log(data);
      }
    });
  }, []);

  return directoryStatus ? (
    <>
      {imagesStatus ? (
        <div className="ml-20 flex min-h-screen mr-4 flex-wrap animate-fade items-center justify-between gap-y-2.5  py-2">
          {imageData.map((data, index) => (
            <BookCover
              className="py-4"
              key={index}
              coverPath={data}
              title={titleData[index]?.title}
            />
          ))}
        </div>
      ) : (
        <></>
      )}
    </>
  ) : (
    <></>
  );
}

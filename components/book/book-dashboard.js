/* eslint-disable camelcase */
import { invoke, convertFileSrc } from "@tauri-apps/api/tauri";
import { useEffect, useState } from "react";
import BookCover from "./book-cover";
import { isValidDirectoryPath } from "@/lib/regex";
import NoDirectory from "../shelf/no-directory";

export default function BookDashboard() {
  const [imageData, setImageData] = useState([]);
  const [titleData, setTitleData] = useState([]);
  const [directoryStatus, setDirectoryStatus] = useState();
  const [imagesStatus, setImagesStatus] = useState();


  useEffect(() => {
    async function loadImages() {
      const start = performance.now();

      const bookCovers = await invoke("initialize_books");
      //const base64ImageAddresses =await invoke("base64_encode_covers");
      const base64ImageAddresses = await Promise.all(
        bookCovers.map(async (book) => {
          return convertFileSrc(book.cover_location);
          // return await invoke("base64_encode_file", {
          //   file_path: book.cover_location,
          // });
        })
      );
      const updateTitleAndImageData = (titles, images) => {
        setTitleData(titles);
        setImageData(images);
      };
      updateTitleAndImageData(bookCovers, base64ImageAddresses);
      const end = performance.now();
      const executionTime = end - start;

      console.log(`Execution time: ${executionTime} milliseconds`);
      setImagesStatus(true);
    }
    invoke("get_configuration_option", {
      option_name: "book_folder_location",
    }).then((data) => {
      if (isValidDirectoryPath(data)) {
        setDirectoryStatus(data);
        loadImages();
      }
    });
  }, []);

  if (imagesStatus) {
    return (
      <>
        {directoryStatus ? (
          <div className="ml-20 flex min-h-screen mr-4 flex-wrap items-center justify-between gap-y-2.5  py-2">
            {imageData.map((data, index) => (
              <BookCover
                className="py-4"
                key={index}
                coverPath={imageData[index]}
                title={titleData[index]?.title}
              />
            ))}
          </div>
        ) : (
          <NoDirectory />
        )}
      </>
    );
  }
  return <div className="min-h-screen"></div>;
}

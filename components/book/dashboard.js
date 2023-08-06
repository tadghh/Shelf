/* eslint-disable camelcase */
import { invoke } from "@tauri-apps/api/tauri";
import { useEffect, useState } from "react";
import BookCover from "./book-cover";
import Link from "next/link";

export default function BookDashboard() {
  const [imageData, setImageData] = useState([]);
  const [titleData, setTitleData] = useState([]);
  const [directoryStatus, setDirectoryStatus] = useState();
  const [imagesStatus, setImagesStatus] = useState();

  useEffect(() => {
    async function loadImages() {
      const start = performance.now();

      const bookCovers = await invoke("create_covers");
      setTitleData(bookCovers);
     // const base64ImageAddresses =await invoke("base64_encode_covers");
      const base64ImageAddresses = await Promise.all(
        bookCovers.map(async (book) => {
          return await invoke("base64_encode_file", {
            file_path: book.cover_location,
          });
        })
      );

      setImageData(base64ImageAddresses);
      const end = performance.now();
      const executionTime = end - start;

      console.log(`Execution time: ${executionTime} milliseconds`);
      setImagesStatus(true);
    }
    invoke("get_configuration_option", {
      option_name: "book_folder_location",
    }).then((data) => {
      if (data != "null" && data != null) {
        setDirectoryStatus(data);
        loadImages();
      }
      setImagesStatus(true);
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
                coverPath={`data:image/jpeg;base64,${data}`}
                title={titleData[index]?.title}
              />
            ))}
          </div>
        ) : (
          <div className="flex justify-center min-h-screen ml-20 ">
            <div className="flex items-center ">
              <div className="flex flex-col justify-around text-black bg-white rounded-md h-4/5 w-80 ">
                <div className="flex flex-col items-center justify-center ">
                  <div className="px-4 py-4 m-2 text-center bg-gray-400 rounded-md">
                    Please configure your book directory:
                  </div>
                  <div className="flex flex-col items-center justify-center ">
                    <Link
                      className="flex items-center justify-center w-20 h-12 underline bg-white rounded-md text-black-500"
                      href="/settings"
                    >
                      Here
                    </Link>
                  </div>
                </div>
                <div className="flex items-center justify-center pb-5 m-2 border-b-2 border-white " />
                <div className="flex items-center justify-center pb-5 ">
                  <div className="flex items-center justify-center border-4 border-white border-dashed rounded-md text-black-500 h-52 w-52">
                    Drop a .epub file here
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}
      </>
    );
  }
  return <div className="min-h-screen"></div>;
}

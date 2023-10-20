/* eslint-disable camelcase */
import { invoke } from "@tauri-apps/api/tauri";
import { useEffect, useState } from "react";
import BookCover from "@/components/book/book-cover";
import { isValidDirectoryPath } from "@/lib/regex";
import NoDirectory from "@/components/shelf/no-directory";

export default function BookDashboard() {
  const [usersBooks, setUsersBooks] = useState([]);
  const [directoryStatus, setDirectoryStatus] = useState(false);
  const [directoryChecked, setDirectoryChecked] = useState(false);
  const [imagesStatus, setImagesStatus] = useState(false);

  useEffect(() => {
    async function loadImages() {
      const start = performance.now();
      setUsersBooks(await invoke("initialize_books"));

      const executionTime = performance.now() - start;

      console.log(`Execution time: ${executionTime} milliseconds`);
      setImagesStatus(true);
    }

    invoke("get_configuration_option", {
      option_name: "book_location",
    }).then((data) => {
      setDirectoryChecked(true);
      if (isValidDirectoryPath(data)) {
        console.log(data);
        setDirectoryStatus(isValidDirectoryPath(data));
        loadImages();
      }
    });
  }, []);
  useEffect(() => {
    console.log(usersBooks);
  }, [usersBooks]);
  if (!directoryChecked) {
    return <></>;
  }

  return directoryStatus && usersBooks != null ? (
    <>
      {imagesStatus ? (
        <div className="ml-20 mr-4 flex min-h-screen animate-fade flex-wrap items-center justify-between gap-y-2.5 py-2">
          {usersBooks.map((book, index) => (
            <BookCover className="py-4" key={index} book={book} />
          ))}
        </div>
      ) : (
        <></>
      )}
    </>
  ) : (
    <NoDirectory />
  );
}

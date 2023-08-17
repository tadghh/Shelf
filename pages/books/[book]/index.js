/* eslint-disable camelcase */
import { invoke } from "@tauri-apps/api/tauri";
import { useRouter } from "next/router";
import { useState, useEffect, useRef } from "react";
import ePub from "epubjs";

export default function Book() {
  const router = useRouter();
  const { book } = router.query;
  const bookRef = useRef(null);
  const [bookData, setBookData] = useState("");
  const [bookOpen, setBookOpen] = useState(false);
  const [bookRender, setBookRender] = useState();
  const [scrollStyle, setScrollStyle] = useState("false");

  const [backgroundData, setBackgroundData] = useState(null);
  const [bookLoaded, setBookLoaded] = useState(false);

  const [viewerHeight, setViewerHeight] = useState(window.innerHeight - 40); // Adjust margin as needed
  const [viewerWidth, setViewerWidth] = useState(window.innerWidth - 40); // Adjust margin as needed
  useEffect(() => {
    const handleResize = () => {
      setViewerHeight(window.innerHeight - 40); // Adjust margin as needed
      setViewerWidth(window.innerWidth - 140); // Adjust margin as needed
    };

    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
    };
  }, []);
  useEffect(() => {
    async function loadBook() {
      if (book !== undefined && !bookOpen) {
        setBookData(await invoke("load_book", { title: book }));

        const bookLoaded = ePub({
          encoding: "base64",
        });

        if (bookData.length !== 0 && !bookLoaded.isOpen) {
          let endlessScrollValue;

          invoke("get_cover", { book_title: book }).then((data) => {
            setBackgroundData(data);
          });

          invoke("get_configuration_option", {
            option_name: "endless_scroll",
          }).then((data) => {
            if (data) {
              endlessScrollValue = data;

              setScrollStyle(endlessScrollValue === "true");
            }
          });

          bookRef.current = bookLoaded;

          bookLoaded.open(bookData);

          try {
            bookLoaded.ready.then(() => {
              //I dont like this null here but nmp atm

              const rendition = bookLoaded.renderTo(
                document.getElementById("viewer"),
                {
                  manager:
                    endlessScrollValue === "true" ? "continuous" : "default",
                  flow: endlessScrollValue === "true" ? "scrolled" : null,
                  width: "100%",
                  height: "80%",
                }
              );

              setBookRender(rendition);
              rendition.display();
            });
          } catch {
            //handle this
          }
          setBookLoaded(true);
          setBookOpen(true);
        }
      }
    }

    loadBook();
  }, [book, bookRef, bookData, bookOpen]);

  useEffect(() => {
    if (typeof window !== "undefined") {
      import("@tauri-apps/api/window").then((module) => {
        const { appWindow } = module;
        appWindow.setTitle(book);
      });
    }

    router.events.on("routeChangeStart", () => {
      if (typeof window !== "undefined") {
        import("@tauri-apps/api/window").then((module) => {
          const { appWindow } = module;
          appWindow.setTitle("Shelf");
        });
      }
    });
  }, [book, router.events]);
  //You can break it by squishing the window to small than it cant scroll

  //div overlaps the book fade to edge with black
  return (
    <>
      {bookLoaded && (
        <div
          className="flex flex-col items-center max-h-screen justify-items-center "
          //   style={{
          //     backgroundImage: `url('data:image/jpeg;base64,${backgroundData}')`,
          //   }}
        >
          <div className="flex flex-col items-center my-5 backdrop-filter backdrop-blur justify-items-center ">
            {scrollStyle ? (
              <div
                id="viewer"
                className="bg-white overflow-hidden ml-20  my-10 rounded w-[800px] h-auto "
              ></div>
            ) : (
              <div
                id="controls"
                className="z-50 flex justify-between ml-20 border rounded-xl max-w-[800px]  "
                style={{
                  height: `${viewerHeight}px`,
                  width: `${viewerWidth}px`,
                }}
              >
                <div
                  onClick={() => bookRender.prev()}
                  className="w-[20px] h-auto px-2 text-xs font-semibold text-gray-900 shadow-sm grow-0 "
                ></div>
                <div
                  id="viewer"
                  className="bg-white "
                  style={{ width: `${viewerWidth}px` }}
                ></div>
                <div
                  onClick={() => bookRender.next()}
                  className="px-2 py-1 w-[20px] text-xs font-semibold text-gray-900 shadow-sm grow-0 "
                ></div>
              </div>
            )}
          </div>
        </div>
      )}
    </>
  );
}

/* eslint-disable camelcase */
import { invoke } from "@tauri-apps/api/tauri";
import { appWindow } from "@tauri-apps/api/window";
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
  const [scrollStyle, setScrollStyle] = useState(false);

  const [bookLoaded, setBookLoaded] = useState(false);

  //calculate the width of the book
  const bookSize = () => {
    return window.innerWidth - 160 > 800 ? 800 : window.innerWidth - 180;
  };
  console.log("Width" + (window.innerWidth - 20)  );
  console.log("Widtdh" + bookSize(0) );
  const [viewerHeight, setViewerHeight] = useState(window.innerHeight - 40);
  const [viewerWidth, setViewerWidth] = useState(bookSize(0));
  useEffect(() => {
    invoke("get_configuration_option", {
      option_name: "endless_scroll",
    }).then((data) => {
      if (data) {


        setScrollStyle(data === "true");
      }
    });
  });
  useEffect(() => {
    const handleResize = () => {
      //less than the max and greater than the min
      setViewerHeight(window.innerHeight - 40);
      setViewerWidth(window.innerWidth - 140);
      if (bookRender) {
        bookRender.resize(bookSize(), window.innerHeight - 40);
      }
    };

    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
    };
  }, [bookRender]);
  useEffect(() => {
    async function loadBook() {
      if (book !== undefined && !bookOpen) {
        setBookData(await invoke("load_book", { title: book }));

        const bookLoaded = ePub({
          encoding: "base64",
        });

        if (bookData.length !== 0 && !bookLoaded.isOpen) {



          bookRef.current = bookLoaded;

          bookLoaded.open(bookData);

          try {
            bookLoaded.ready.then(() => {
              //I dont like this null here but nmp atm
              let bookWidth = bookSize() + "";

              const rendition = bookLoaded.renderTo(
                document.getElementById("viewer"),
                {
                  manager:
                  scrollStyle  ? "continuous" : "default",
                  flow: scrollStyle ? "scrolled" : null,
                  width: bookWidth,
                  height: window.innerHeight - 40,
                  spread: "none",
                }
              );
              // rendition.resize(800, window.innerHeight - 40 );
              console.log(rendition);
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
      appWindow.setTitle(book);
    }

    router.events.on("routeChangeStart", () => {
      appWindow.setTitle("Shelf");
    });
  }, [book, router.events]);
  //You can break it by squishing the window to small than it cant scroll

  //div overlaps the book fade to edge with black
  return (
    <>
      {bookLoaded && (
        <div
          className="flex flex-col items-center max-h-screen justify-items-center "

        >
          <div className="flex flex-col items-center my-5 ml-20 backdrop-filter backdrop-blur justify-items-center " style={{
                  height: `${viewerHeight}px`,
                  width: `${viewerWidth}px`,
                }}>
            {scrollStyle ? (
              <div
                id="viewer"
                className="bg-white rounded-xl overflow-clip  max-w-[800px]  "

              />
            ) : (
              <div
                id="controls"
                className="z-40 flex justify-between border rounded-xl max-w-[840px]  overflow-hidden"

              >
                <div
                  onClick={() => bookRender.prev()}
                  className="w-[20px] h-auto px-2 text-xs font-semibold bg-gradient-to-r from-black to-white text-gray-900 shadow-sm grow-0 "
                ></div>
                <div
                  id="viewer"
                  className="bg-white "

                />
                <div
                  onClick={() => bookRender.next()}
                  className="px-2 py-1 w-[20px] z-50 text-xs font-semibold text-gray-900 bg-gradient-to-l from-black to-white shadow-sm grow-0 "
                ></div>
              </div>
            )}
          </div>
        </div>
      )}
    </>
  );
}
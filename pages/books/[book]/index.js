/* eslint-disable camelcase */
import { invoke } from "@tauri-apps/api/tauri";
import { appWindow } from "@tauri-apps/api/window";
import { useRouter } from "next/router";
import { useState, useEffect, useRef } from "react";
import ePub from "epubjs";

export default function Book() {
  const router = useRouter();
  const bookRef = useRef();

  const { book } = router.query;

  const [bookData, setBookData] = useState();
  const [bookOpen, setBookOpen] = useState(false);
  const [bookLoaded, setBookLoaded] = useState(false);
  const [bookRender, setBookRender] = useState();

  const [scrollStyle, setScrollStyle] = useState(false);

  //calculate the width of the book without margins
  const bookSize = () => {
    return window.innerWidth - 160 > 800 ? 800 : window.innerWidth - 180;
  };

  const [viewerHeight, setViewerHeight] = useState(window.innerHeight - 40);
  const [viewerWidth, setViewerWidth] = useState(bookSize());

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
      setBookData(await invoke("load_book", { title: book }));

      const bookLoaded = ePub({
        encoding: "base64",
      });

      if (bookData && !bookLoaded.isOpen) {
        bookRef.current = bookLoaded;

        bookLoaded.open(bookData);

        try {
          bookLoaded.ready.then(() => {
            let bookWidth = bookSize() + "";
            let settings = {
              width: bookWidth,
              height: window.innerHeight - 40,
              spread: "none",
            };

            scrollStyle
              ? () => {
                  settings.manager = "continuous";
                  settings.flow = "scrolled";
                }
              : (settings.manager = "default");

            const rendition = bookLoaded.renderTo(
              document.getElementById("viewer"),
              settings
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
    if (book !== undefined && !bookOpen) {
      loadBook();
    }
  }, [book, bookRef, bookData, bookOpen]);

  useEffect(() => {
    if (typeof window !== "undefined") {
      appWindow.setTitle(book);
    }

    router.events.on("routeChangeStart", () => {
      appWindow.setTitle("Shelf");
    });
  }, [book, router.events]);

  return (
    <>
      {bookLoaded && (
        <div className="flex flex-col items-center max-h-screen justify-items-center">
          <div
            className="flex flex-col items-center my-5 ml-20 backdrop-filter backdrop-blur justify-items-center "
            style={{
              height: `${viewerHeight}px`,
              width: `${viewerWidth}px`,
            }}
          >
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
                <div id="viewer" className="bg-white " />
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

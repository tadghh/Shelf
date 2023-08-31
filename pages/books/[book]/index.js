/* eslint-disable camelcase */
import { invoke } from "@tauri-apps/api/tauri";
import { appWindow } from "@tauri-apps/api/window";
import { useRouter } from "next/router";
import { useState, useEffect, useRef, useCallback } from "react";
import ePub from "epubjs";
import PageButton from "@/components/book/page-button";

export default function Book() {
  const router = useRouter();
  const bookRenderRef = useRef(); // Create a ref for bookRender
  const bookLoadRef = useRef(); // Create a ref for bookRender
  const isLoadBookCalledRef = useRef(false);

  const { book } = router.query;

  const [bookOpen, setBookOpen] = useState(false);
  const [bookRender, setBookRender] = useState();

  const [scrollStyle, setScrollStyle] = useState(false);

  //calculate the width of the book without margins
  const bookSize = () => {
    return window.innerWidth - 160 > 800 ? 800 : window.innerWidth - 180;
  };

  const [viewerHeight, setViewerHeight] = useState(window.innerHeight - 40);
  const [viewerWidth, setViewerWidth] = useState(bookSize());
  const handlePrevPage = useCallback(() => {
    if (bookRenderRef.current) {
      bookRenderRef.current.prev();
    }
  }, []);

  const handleNextPage = useCallback(() => {
    if (bookRenderRef.current) {
      console.log(bookRenderRef.current.next());
      bookRenderRef.current.next();
    }
  }, []);
  useEffect(() => {
    async function usersBookSettings() {
      await invoke("get_configuration_option", {
        option_name: "endless_scroll",
      }).then((data) => {
        if (data) {
          setScrollStyle(data === "true");
        }
      });
    }
    usersBookSettings();
  }, []);

  const getBookSettings = async () => {
    let scrollValue = false;
    await invoke("get_configuration_option", {
      option_name: "endless_scroll",
    }).then((data) => {
      if (data) {
        scrollValue = data === "true";
      }
    });
    return scrollValue;
  };
  useEffect(() => {
    const handleResize = () => {
      setViewerHeight(window.innerHeight - 40);
      setViewerWidth(window.innerWidth - 140);
      if (bookRenderRef.current) {
        bookRenderRef.current.resize(bookSize(), window.innerHeight - 40);
      }
    };

    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
    };
  }, []);

  useEffect(() => {
    if (book && !isLoadBookCalledRef.current) {
      isLoadBookCalledRef.current = true;

      invoke("load_book", { title: book }).then(async (bookData) => {
        if (bookData) {
          bookLoadRef.current = ePub({
            encoding: "base64",
          });

          if (!bookLoadRef.current.isOpen) {
            bookLoadRef.current.open(bookData);

            try {
              await bookLoadRef.current.ready;

              let bookWidth = bookSize();
              const scrollValue = await getBookSettings();

              let settings = {
                width: bookWidth,
                height: window.innerHeight - 40,
                spread: "none",
              };

              if (scrollValue) {
                settings.manager = "continuous";
                settings.flow = "scrolled";
              } else {
                settings.manager = "default";
              }

              const rendition = bookLoadRef.current.renderTo(
                document.getElementById("viewer"),
                settings
              );

              bookRenderRef.current = rendition;

              rendition.display();
            } catch {
              //handle this
            }
          }
        }
      });
    }
  }, []);

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
      {true && (
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
                <PageButton action={handlePrevPage} left />
                <div id="viewer" className="bg-white " />
                <PageButton action={handleNextPage} />
              </div>
            )}
          </div>
        </div>
      )}
    </>
  );
}

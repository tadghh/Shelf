/* eslint-disable camelcase */
import { invoke, convertFileSrc } from "@tauri-apps/api/tauri";
import { appWindow } from "@tauri-apps/api/window";
import { useRouter } from "next/router";
import { useState, useEffect, useRef, useCallback } from "react";
import ePub from "epubjs";

import PageButton from "@/components/book/page-button";
import { SettingsItems } from "@/lib/SettingsItemEnum";

export default function Book() {
  const router = useRouter();

  const bookRenderRef = useRef();
  const bookBackgroundRef = useRef();
  const bookLoadRef = useRef();
  const coverBackgroundState = useRef();
  const scrollStyleState = useRef();

  const settingsEnums = useRef();

  const isLoadBookCalledRef = useRef(false);

  const { book } = router.query;

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

  //Maybe we put this into its own file
  async function loadEnum() {
    settingsEnums.current = await SettingsItems();
  }

  async function usersBookSettings() {
    await loadEnum();

    await invoke("get_configuration_option", {
      option_name: settingsEnums.current.ENDLESS_SCROLL,
    }).then((data) => {
      if (data) {
        setScrollStyle(data === "true");
        scrollStyleState.current = data === "true";
      }
    });

    await invoke("get_configuration_option", {
      option_name: settingsEnums.current.COVER_BACKGROUND,
    }).then((data) => {
      if (data) {
        coverBackgroundState.current = data === "true";
      }
    });
  }

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
    async function loadBook() {
      await usersBookSettings();

      if (book && !isLoadBookCalledRef.current) {
        isLoadBookCalledRef.current = true;

        invoke("load_book", { title: book }).then(async (bookInfo) => {
          if (bookInfo) {
            bookLoadRef.current = ePub();

            if (!bookLoadRef.current.isOpen) {
              bookLoadRef.current.open(convertFileSrc(bookInfo.book_location));

              if (
                bookBackgroundRef.current &&
                coverBackgroundState.current === true
              ) {
                bookBackgroundRef.current.style.backgroundImage = `url(${convertFileSrc(
                  bookInfo.cover_location
                )})`;
              }
              try {
                await bookLoadRef.current.ready;

                let bookWidth = bookSize();
                const scrollValue = scrollStyleState.current;

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
    }
    loadBook();
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
        <div
          className="max-h-screen bg-gray-500 bg-center bg-cover "
          ref={bookBackgroundRef}
        >
          <div className="flex flex-col items-center w-full h-full backdrop-blur-sm backdrop-brightness-50">
            <div
              className="z-50 flex flex-col items-center my-5 ml-20 opacity-100 justify-items-center "
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
        </div>
      )}
    </>
  );
}

/* eslint-disable camelcase */
import { invoke, convertFileSrc } from "@tauri-apps/api/tauri";
import { useRouter } from "next/router";
import { useState, useEffect, useRef, useCallback } from "react";
import ePub from "epubjs";

import PageButton from "@/components/book/page-button";
import { SettingsItems } from "@/lib/SettingsItemEnum";

export default function Book() {
  const VERTICAL_PADDING = 40;
  const MAX_WIDTH = 800;
  const router = useRouter();

  const [viewerHeight, setViewerHeight] = useState(null);
  const [viewerWidth, setViewerWidth] = useState(null);

  const bookRenderRef = useRef();
  const handlePrevPage = useCallback(
    () => bookRenderRef.current && bookRenderRef.current.prev(),
    [],
  );
  const handleNextPage = useCallback(
    () => bookRenderRef.current && bookRenderRef.current.next(),
    [],
  );

  const bookBackgroundRef = useRef();
  const bookLoadRef = useRef();
  const coverBackgroundState = useRef();
  const scrollStyleState = useRef();

  const settingsEnums = useRef();

  const isLoadBookCalledRef = useRef(false);

  const { book } = router.query;

  const [scrollStyle, setScrollStyle] = useState(false);

  //calculate the width of the book without margins
  const getWidth = () => {
    if (typeof window !== "undefined") {
      return window.innerWidth - 160 > MAX_WIDTH
        ? MAX_WIDTH
        : window.innerWidth - 160;
    }
  };
  const getHeight = () => {
    if (typeof window !== "undefined") {
      return window.innerHeight - VERTICAL_PADDING;
    }
  };

  useEffect(() => {
    if (typeof window !== "undefined") {
      setViewerHeight(getHeight());
      setViewerWidth(getWidth());
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
      setViewerHeight(getHeight());
      setViewerWidth(getWidth());
      if (bookRenderRef.current) {
        bookRenderRef.current.resize(getWidth(), getHeight());
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
                  bookInfo.cover_location,
                )})`;
              }
              try {
                await bookLoadRef.current.ready;

                let bookWidth = getWidth();
                const scrollValue = scrollStyleState.current;

                let settings = {
                  width: bookWidth,
                  height: getHeight(),
                  spread: "none",
                };

                if (scrollValue) {
                  settings.manager = "continuous";
                  settings.flow = "scrolled";
                } else {
                  settings.manager = "default";
                }

                bookRenderRef.current = bookLoadRef.current.renderTo(
                  document.getElementById("viewer"),
                  settings,
                );

                bookRenderRef.current.display();
              } catch {
                //handle this
                //no :P
              }
            }
          }
        });
      }
    }
    loadBook();
  }, [router.isReady]);

  useEffect(() => {
    //Import on run
    // if (typeof window !== "undefined") {
    //   appWindow.setTitle(book);
    // }
    // router.events.on("routeChangeStart", () => {
    //   appWindow.setTitle("Shelf");
    // });
  }, [book, router.events]);
  return (
    <div
      className="max-h-screen bg-gray-500 bg-cover bg-center "
      ref={bookBackgroundRef}
    >
      <div className="flex h-full w-full flex-col items-center backdrop-blur-sm backdrop-brightness-50">
        <div
          className="z-50 my-5 ml-20 flex flex-col items-center justify-items-center opacity-100 "
          style={{
            height: `${viewerHeight}px`,
            width: `${viewerWidth}px`,
          }}
        >
          {scrollStyle ? (
            <div
              id="viewer"
              className="max-w-[800px] overflow-clip rounded-xl  bg-white  "
            />
          ) : (
            <div
              id="controls"
              className="z-40 flex max-w-[840px] justify-between overflow-hidden rounded-xl  border"
            >
              <PageButton action={handlePrevPage} left />
              <div id="viewer" className="bg-white " />
              <PageButton action={handleNextPage} />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

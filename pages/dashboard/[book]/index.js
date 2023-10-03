/* eslint-disable camelcase */
import { invoke, convertFileSrc } from "@tauri-apps/api/tauri";
import { useRouter } from "next/router";
import { useState, useEffect, useRef, useCallback } from "react";
import ePub from "epubjs";

import PageButton from "@/components/book/page-button";
import { SettingsItems } from "@/lib/SettingsItemEnum";

export default function Book() {
  const VERTICAL_PADDING = 40;
  const WIDTH_PADDING = 160;
  const MAX_WIDTH = 800;
  const router = useRouter();

  const [viewerHeight, setViewerHeight] = useState(null);
  const [viewerWidth, setViewerWidth] = useState(null);

  const bookRender = useRef();
  const handlePrevPage = useCallback(
    () => bookRender.current && bookRender.current.prev(),
    [],
  );
  const handleNextPage = useCallback(
    () => bookRender.current && bookRender.current.next(),
    [],
  );

  const bookEpub = useRef();

  const coverBackgroundState = useRef();
  const bookBackgroundUrl = useRef();

  const scrollStyleState = useRef();
  const [scrollStyle, setScrollStyle] = useState(false);

  const settingsEnums = useRef();

  const isBookLoaded = useRef(false);

  const { book } = router.query;

  //calculate the width of the book without margins
  const getWidth = () => {
    if (typeof window !== "undefined") {
      return window.innerWidth - WIDTH_PADDING > MAX_WIDTH
        ? MAX_WIDTH
        : window.innerWidth - WIDTH_PADDING;
    }
  };
  const getHeight = () => {
    if (typeof window !== "undefined") {
      return window.innerHeight - VERTICAL_PADDING;
    }
  };
  //Maybe we put this into its own file
  async function loadEnum() {
    settingsEnums.current = await SettingsItems();
  }
  async function usersBookSettings() {
    await loadEnum();

    await invoke("get_configuration_option", {
      option_name: settingsEnums.current.ENDLESS_SCROLL,
    }).then((data) => {
      console.log(data);
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
    if (typeof window !== "undefined") {
      setViewerHeight(getHeight());
      setViewerWidth(getWidth());
    }
  }, []);

  //Loads the book. Check if the router isReady... fixes refreshing refs on page refresh
  useEffect(() => {
    async function loadBook() {
      await usersBookSettings();

      if (book && !isBookLoaded.current) {
        isBookLoaded.current = true;

        invoke("load_book", { title: book }).then(async (bookInfo) => {
          if (bookInfo) {
            bookEpub.current = ePub();

            if (!bookEpub.current.isOpen) {
              bookEpub.current.open(convertFileSrc(bookInfo.book_location));

              if (
                bookBackgroundUrl.current &&
                coverBackgroundState.current === true
              ) {
                bookBackgroundUrl.current.style.backgroundImage = `url(${convertFileSrc(
                  bookInfo.cover_location,
                )})`;
              }
              try {
                await bookEpub.current.ready;

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

                //duplicated?
                bookRender.current = bookEpub.current.renderTo(
                  document.getElementById("viewer"),
                  settings,
                );

                bookRender.current.display();
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
    const handleResize = () => {
      setViewerHeight(getHeight());
      setViewerWidth(getWidth());
      if (bookRender.current) {
        bookRender.current.resize(getWidth(), getHeight());
      }
    };

    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
    };
  }, []);

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
    <div className="max-h-screen bg-center bg-cover " ref={bookBackgroundUrl}>
      <div className="flex flex-col items-center w-full h-full transition-opacity duration-100 ease-out animate-fade backdrop-blur-sm backdrop-brightness-50 ">
        <div
          className="z-50 flex flex-col items-center my-5 ml-20 bg-white opacity-100 justify-items-center "
          style={{
            height: `${viewerHeight}px`,
            width: `${viewerWidth}px`,
          }}
        >
          {scrollStyle ? (
            <div
              id="viewer"
              className="max-w-[800px] overflow-clip rounded-xl  "
            />
          ) : (
            <div
              id="controls"
              className={
                "z-40 max-w-[840px] justify-between overflow-hidden " +
                bookRender.current
                  ? "flex"
                  : "hidden"
              }
            >
              <PageButton action={handlePrevPage} left />
              <div id="viewer" />
              <PageButton action={handleNextPage} />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

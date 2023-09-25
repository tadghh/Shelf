/* eslint-disable camelcase */
import { invoke, convertFileSrc } from "@tauri-apps/api/tauri";
import { useRouter } from "next/router";
import { useState, useEffect, useRef, useCallback } from "react";
import ePub from "epubjs";

import PageButton from "@/components/book/page-button";
import { SettingsItems } from "@/lib/SettingsItemEnum";

export default function Book() {
  const router = useRouter();
  const [bookName, setBookName] = useState();

  const bookRenderRef = useRef();
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

  const bookBackgroundRef = useRef();
  const bookLoadRef = useRef();
  const coverBackgroundState = useRef();
  const scrollStyleState = useRef();

  const settingsEnums = useRef();

  const isLoadBookCalledRef = useRef(false);

  const { book } = router.query;
  useEffect(() => {
    if (router.query.book != undefined) {
      setBookName(router.query.book);
      console.log("yo");
      console.log(router.query.book);
    }
  }, [router.query.book]);
  const [scrollStyle, setScrollStyle] = useState(false);

  //calculate the width of the book without margins
  const bookSize = () => {
    if (typeof window !== "undefined") {
      return window.innerWidth - 160 > 800 ? 800 : window.innerWidth - 180;
    } else {
      // Return a default value or handle the case where window is not defined.
      return 682; // You can choose a suitable default value.
    }
  };
  const getHeight = () => {
    if (typeof window !== "undefined") {
      if (bookRenderRef.current) {
        bookRenderRef.current.resize(bookSize(), 1017);
      }
      return window.innerHeight - 40;
    } else {
      // Return a default value or handle the case where window is not defined.
      return 734;
      // if (bookRenderRef.current) {
      //   bookRenderRef.current.resize(bookSize(), 1017);
      // }
    }
  };

  const [viewerHeight, setViewerHeight] = useState(getHeight());
  const [viewerWidth, setViewerWidth] = useState(bookSize());

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
      console.log("reload");
      console.log("Current book" + book);
      console.log("Current book" + bookName);
      console.log("Current book" + router.query);
      console.log(router.query.book);
      console.log(await router.query);
      console.log("Current ref" + isLoadBookCalledRef.toString());

      console.log(!isLoadBookCalledRef.current);

      if (book && !isLoadBookCalledRef.current) {
        isLoadBookCalledRef.current = true;
        console.log("loading of book?");

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
                  settings,
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
    <>
      {true && (
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
      )}
    </>
  );
}

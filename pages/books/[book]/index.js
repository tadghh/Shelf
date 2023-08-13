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
					}).then(data => {
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
									manager: endlessScrollValue === "true" ? "continuous" : "default",
									flow: endlessScrollValue === "true" ? "scrolled" : null,
									width: "100%",
									height: "100%",
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

		router.events.on('routeChangeStart', () => {
			if (typeof window !== "undefined") {
				import("@tauri-apps/api/window").then((module) => {
					const { appWindow } = module;
					appWindow.setTitle("Shelf");
				});
			}
		});


	}, [book, router.events]);
	//You can break it by squishing the window to small than it cant scroll
	return (
		<>
			{bookLoaded &&
				<div className="flex flex-col items-center max-h-screen justify-items-center " style={{ backgroundImage: `url('data:image/jpeg;base64,${backgroundData}')` }}>
					<div className="flex flex-col items-center w-full h-full overflow-hidden backdrop-filter backdrop-blur justify-items-center " >
						<div id="viewer" className="bg-white overflow-hidden ml-20 border-2 border-orange-700 my-10 rounded-lg w-[800px] h-[1800px]"  >

							{scrollStyle ? null :
								<div id="controls" className="flex justify-between">
									<button
										onClick={() => bookRender.prev()}
										type="button"
										className="px-2 py-1 text-xs font-semibold text-gray-900 bg-white rounded shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50"
									>
										Previous
									</button>
									<button
										onClick={() => bookRender.next()}
										type="button"
										className="px-2 py-1 text-xs font-semibold text-gray-900 bg-white rounded shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50"
									>
										Next
									</button>
								</div>
							}

						</div>
					</div>

				</div>
			}
		</>
	);
}

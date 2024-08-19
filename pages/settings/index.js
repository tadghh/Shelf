/* eslint-disable no-unused-vars */
/* eslint-disable camelcase */
import { open } from "@tauri-apps/api/dialog";

import { invoke } from "@tauri-apps/api/tauri";
import { useEffect, useState } from "react";
import SettingsItem from "@/components/settings/settings-item";
import { SettingsTypes } from "@/lib/SettingsTypeEnum";
import { SettingsItems } from "@/lib/SettingsItemEnum";
import useNotification from "@/lib/notifications/notificationHook";
import { notificationState } from "@/lib/notifications/notificationStates";

export default function Settings() {
  const [settingsItemsEnum, setSettingsItemsEnum] = useState([]);
  const [rerenderKey, setRerenderKey] = useState(0);
  const { notify } = useNotification();

  const updateSettingsItems = async () => {
    setSettingsItemsEnum(await SettingsItems());
  };

  useEffect(() => {
    // TODO what is this?
    updateSettingsItems();
  }, []);

  const resetHandler = () => {
    invoke("reset_configuration")
      .then(() => {
        setRerenderKey((prevKey) => prevKey + 1);
        notify(
          notificationState.SUCCESS,
          "Settings and cache reset successfully.",
        );
      })
      .catch((error) => {
        notify(notificationState.ERROR, "An error occurred when resetting.");
      });
  };
  const importOldHandler = (data) => {
    invoke("import_book_json_comm", { backup_path: data })
      .then(() => {
        notify(notificationState.SUCCESS, "Imported old books successfully.");
      })
      .catch((error) => {
        notify(
          notificationState.ERROR,
          `An error occurred while importing the old books.  ${error}`,
        );
      });
  };
  const exportCurrentDBHandler = (data) => {
    invoke("backup_books_to_json", { path: data })
      .then(() => {
        notify(
          notificationState.SUCCESS,
          "Exported current books successfully.",
        );
      })
      .catch((error) => {
        notify(
          notificationState.ERROR,
          `An error occurred while exporting the current books. ${error}`,
        );
      });
  };
  return (
    <div className="duration-550 ml-20 min-h-screen animate-fade flex-col px-5 py-2 transition-opacity ease-in-out">
      {settingsItemsEnum.length != 0 ? (
        <>
          <SettingsItem
            key={`item1-${rerenderKey}`}
            settingsTitle="Book directory"
            settingsDescription="The folder containing your books"
            settingsConfigString={settingsItemsEnum.BOOK_LOCATION}
            settingsType={SettingsTypes.FILE}
          />
          <SettingsItem
            key={`item2-${rerenderKey}`}
            settingsTitle="Endless scrolling"
            settingsDescription="The next page will load as you scroll"
            settingsConfigString={settingsItemsEnum.ENDLESS_SCROLL}
            settingsType={SettingsTypes.TOGGLE}
          />
          <SettingsItem
            key={`item3-${rerenderKey}`}
            settingsTitle="Book cover as background"
            settingsDescription="Uses the books cover image as a background"
            settingsConfigString={settingsItemsEnum.COVER_BACKGROUND}
            settingsType={SettingsTypes.TOGGLE}
          />
          <div className="flex w-full justify-between">
            <div className="flex justify-between space-x-4">
              <div className="mt-2 flex h-16 w-44 items-center justify-center rounded-xl border bg-white p-4">
                <button
                  className="font-sm rounded-lg border-4 border-white bg-red-700 px-5 py-1 text-sm font-bold text-white transition-colors duration-300 ease-in-out hover:border-red-500 hover:bg-red-800"
                  type="button"
                  onClick={() => {
                    open({
                      directory: false,
                      multiple: false,
                    }).then((data) => {
                      if (data) {
                        importOldHandler(data);
                      }
                    });
                  }}
                >
                  Import old books
                </button>
              </div>
              <div className="mt-2 flex h-16 w-44 items-center justify-center rounded-xl border bg-white p-4">
                <button
                  className="font-sm rounded-lg border-4 border-white bg-yellow-700 px-5 py-1 text-sm font-bold text-white transition-colors duration-300 ease-in-out hover:border-yellow-500 hover:bg-yellow-800"
                  type="button"
                  onClick={() => {
                    open({
                      directory: true,
                      multiple: false,
                    }).then((data) => {
                      if (data) {
                        exportCurrentDBHandler(data);
                      }
                    });
                  }}
                >
                  Export current books
                </button>
              </div>
            </div>
            <div className="mt-2 flex h-16 w-44 items-center justify-center rounded-xl border bg-white p-4">
              <button
                className="rounded-lg border-4 border-white bg-red-700 px-5 py-2.5 text-sm font-medium text-white transition-colors duration-300 ease-in-out hover:border-red-500 hover:bg-red-800"
                type="button"
                onClick={resetHandler}
              >
                Reset settings
              </button>
            </div>
          </div>
        </>
      ) : (
        <>
          <span>Failed to load the settings menu</span>
        </>
      )}
    </div>
  );
}

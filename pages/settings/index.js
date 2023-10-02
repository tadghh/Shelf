/* eslint-disable no-unused-vars */
/* eslint-disable camelcase */

import { invoke } from "@tauri-apps/api/tauri";
import { useEffect, useState } from "react";
import SettingsItem from "@/components/settings/settings-item";
import { SettingsTypes } from "@/lib/SettingsTypeEnum";

export default function Settings() {
  const [settingsItemsEnum, setSettingsItemsEnum] = useState([]);

  useEffect(() => {
    invoke("shelf_settings_values").then((data) => {
      setSettingsItemsEnum(data);
    });
  }, []);
  const resetHandler = () => {
    invoke("reset_configuration");
  };
  return (
    <div className="duration-550 ml-20 min-h-screen animate-fade flex-col px-5 py-2 transition-opacity ease-in-out">
      {settingsItemsEnum.length != 0 ? (
        <>
          <SettingsItem
            settingsTitle="Book directory"
            settingsDescription="The folder containing your books"
            settingsConfigString={settingsItemsEnum.BOOK_LOCATION}
            settingsType={SettingsTypes.FILE}
          />
          <SettingsItem
            settingsTitle="Endless scrolling"
            settingsDescription="The next page will load as you scroll"
            settingsConfigString={settingsItemsEnum.ENDLESS_SCROLL}
            settingsType={SettingsTypes.TOGGLE}
          />
          <SettingsItem
            settingsTitle="Book cover as background"
            settingsDescription="Uses the books cover image as a background"
            settingsConfigString={settingsItemsEnum.COVER_BACKGROUND}
            settingsType={SettingsTypes.TOGGLE}
          />
          <div className="ml-auto mt-2 flex h-16 w-44 items-center justify-center rounded-xl border bg-white p-4">
            <button
              className=" rounded-lg bg-red-700 px-5 py-2.5 text-sm font-medium text-white hover:bg-red-800 focus:outline-none focus:ring-4 focus:ring-red-300 dark:bg-red-600 dark:hover:bg-red-700 dark:focus:ring-red-900"
              type="button"
              onClick={resetHandler}
            >
              Reset settings
            </button>
          </div>
        </>
      ) : (
        <></>
      )}
    </div>
  );
}

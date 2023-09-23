import { invoke } from "@tauri-apps/api";

import SettingsItem from "@/components/settings/settings-item";
import { SettingsTypes } from "@/lib/SettingsTypeEnum";
import { useEffect, useState } from "react";

export default function Settings() {
  const [settingsItemsEnum, setSettingsItemsEnum] = useState([]);

  useEffect(() => {
    // invoke("get_configuration_option", {
    //   option_name: "book_location",
    // }).then((data) => {
    //   setSettingsItemsEnum(data);
    // });
  }, []);
  return <></>;
  return (
    <div className="duration-550 ml-20 min-h-screen animate-fade flex-col px-5 py-2 transition-opacity ease-in-out">
      {/* {settingsItemsEnum.length != 0 ? (
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
          <div className="flex items-center justify-center h-16 p-4 mt-2 ml-auto bg-white border w-44 rounded-xl">
            <button
              className=" rounded-lg bg-red-700 px-5 py-2.5 text-sm font-medium text-white hover:bg-red-800 focus:outline-none focus:ring-4 focus:ring-red-300 dark:bg-red-600 dark:hover:bg-red-700 dark:focus:ring-red-900"
              type="button"
              onClick={resetSettings}
            >
              Reset settings
            </button>
          </div>
        </>
      ) : (
        <></>
      )} */}
    </div>
  );
}

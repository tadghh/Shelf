/* eslint-disable no-unused-vars */
/* eslint-disable camelcase */

import { invoke } from "@tauri-apps/api/tauri";
import { useEffect, useState } from "react";
import SettingsItem from "@/components/settings/settings-item";
import { SettingsTypes } from "@/lib/SettingsTypeEnum";
import { SettingsItems } from "@/lib/SettingsItemEnum";

export default function Settings() {
  const [settingsItemsEnum, setSettingsItemsEnum] = useState([]);
  const [rerenderKey, setRerenderKey] = useState(0);

  const updateSettingsItems = async () => {
    setSettingsItemsEnum(await SettingsItems());
  };

  useEffect(() => {
    updateSettingsItems();
  }, []);

  const resetHandler = () => {
    invoke("reset_configuration")
      .then(() => {
        setRerenderKey((prevKey) => prevKey + 1);
      })
      .catch((error) => {
        console.error("Error resetting configuration:", error);
      });
  };
  return (
    <div className="flex-col min-h-screen px-5 py-2 ml-20 transition-opacity ease-in-out duration-550 animate-fade">
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
          <div className="flex items-center justify-center h-16 p-4 mt-2 ml-auto bg-white border w-44 rounded-xl">
            <button
              className=" rounded-lg  border-4 border-white bg-red-700 px-5 py-2.5 text-sm font-medium text-white transition-colors duration-300 ease-in-out hover:border-red-500 hover:bg-red-800"
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

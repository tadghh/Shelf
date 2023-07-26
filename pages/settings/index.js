/* eslint-disable camelcase */
import SettingsItem from "@/components/settings/settings-item";
import { SettingsTypes } from "@/lib/SettingsTypeEnum";
import { SettingsItems } from "@/lib/SettingsItemEnum";
import { useEffect, useState } from "react";

export default function Settings() {
  const [settingsItemsEnum, setSettingsItemsEnum] = useState();
  useEffect(() => {
    async function loadEnum() {
      setSettingsItemsEnum(await SettingsItems());
    }
    loadEnum();
  }, []);

  return (
    <div
      className={`flex-col min-h-screen px-5 py-2 ml-20 transition-opacity duration-550 ease-in-out ${
        settingsItemsEnum ? "opacity-100" : "opacity-0"
      }`}
    >
      {settingsItemsEnum ? (
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
        </>
      ) : (
        <></>
      )}
    </div>
  );
}

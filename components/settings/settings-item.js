/* eslint-disable no-unused-vars */
/* eslint-disable react-hooks/exhaustive-deps */
/* eslint-disable camelcase */
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { getComponentForEnum } from "@/lib/SettingsTypeReturn";
export default function SettingsItem({
  settingsTitle,
  settingsConfigString,
  settingsType,
  settingsDescription = "",
}) {
  const [settingsItemStatus, setSettingsItemStatus] = useState("");
  const [settingsItemLoaded, setSettingsItemLoaded] = useState(false);
  const Component = getComponentForEnum(settingsType);
  console.log(settingsConfigString);
  const updateOption = ({ value }) => {
    invoke("change_configuration_option", {
      option_name: settingsConfigString,
      value: value + "",
    }).then(setSettingsItemStatus(value+""));
  };

  useEffect(() => {
    invoke("get_configuration_option", {
      option_name: settingsConfigString,
    }).then((data) => {
      if (data) {
        console.log("loaded" + settingsConfigString);
        console.log("loaded" + data);
        console.log(typeof data);
        setSettingsItemStatus(data);
        setSettingsItemLoaded(true);
      }
    });
  }, []);
  useEffect(() => {
    console.log(typeof settingsItemStatus);
    console.log( settingsItemStatus);
    console.log( settingsItemStatus != "");
  }, [settingsItemStatus]);
  return settingsItemStatus != ""? (
    <div className="flex items-center justify-between w-full p-4 mt-2 bg-white border h-28 rounded-xl">
      <div className="flex text-gray-900 ">
        <h2 className="pr-2 text-2xl font-bold leading-4 ">{settingsTitle}</h2>
        <p> {settingsDescription}</p>
      </div>
      <form>
        {Component && (
          <Component
            setter={(value) => updateOption({ value })}
            status={settingsItemStatus}
          />
        )}
      </form>
    </div>
  ) : <></> ;
}

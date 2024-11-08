/* eslint-disable no-unused-vars */
/* eslint-disable react-hooks/exhaustive-deps */
/* eslint-disable camelcase */
import { invoke } from "@tauri-apps/api/tauri";
import { useState, useEffect } from "react";
import { getComponentForEnum } from "@/lib/SettingsTypeReturn";

export default function SettingsItem({
  settingsTitle,
  settingsConfigString,
  settingsType,
  settingsDescription = "",
}) {
  const [settingsItemStatus, setSettingsItemStatus] = useState("");
  const Component = getComponentForEnum(settingsType);

  // TODO fix this sh
  const updateOption = ({ value }) => {
    invoke("change_configuration_option", {
      option_name: settingsConfigString,
      value: value + "",
    }).then(setSettingsItemStatus((value += "")));
  };

  useEffect(() => {
    invoke("get_configuration_option", {
      option_name: settingsConfigString,
    }).then((data) => {
      if (data) {
        setSettingsItemStatus(data);
      }
    });
  }, []);

  return settingsItemStatus != "" ? (
    <div className="mt-2 flex h-28 w-full items-center justify-between rounded-xl border bg-white p-4">
      <div className="flex text-gray-900">
        <h2 className="pr-2 text-2xl font-bold leading-4">{settingsTitle}</h2>
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
  ) : (
    <></>
  );
}

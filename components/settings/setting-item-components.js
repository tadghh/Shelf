import { open } from "@tauri-apps/api/dialog";
import ToggleButton from "../Inputs/toggle-button";
import { useEffect } from "react";
const FileSettingComponent = ({ setter, status = "" }) => {
  return (
    <span
      className="flex-none rounded-md bg-white border-indigo-600 border-2 px-5 py-2.5 text-sm font-semibold text-blue-500 shadow-sm"
      onClick={() => {
        open({
          directory: true,
          multiple: false,
        }).then((data) => {
          if (data) {
            setter(data);
          }
        });
      }}
    >
      {status}
    </span>
  );
};
const ToggleSettingComponent = ({ setter, status }) => {
  //This is bad and should be rewritten
  if (status == "true") {
    status = true;
  } else if (status == "false") {
    status = false;
  }

  return (
    <div onClick={() => setter(!status)}>
      <ToggleButton state={status} />
    </div>
  );
};

export { FileSettingComponent, ToggleSettingComponent };

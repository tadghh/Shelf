import { open } from "@tauri-apps/api/dialog";
import ToggleButton from "../Inputs/toggle-button";
const FileSettingComponent = ({ setter, status = "" }) => {
  return (
    <div
      className={
        "location-input whitespace-pre rounded-md border-2 px-5 py-2.5 " +
        (status != "" ? "border-green-600" : "border-red-700")
      }
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
      <span className="flex-none font-semibold text-blue-500 ">{status}</span>
    </div>
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

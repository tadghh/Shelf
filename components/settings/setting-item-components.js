import { open } from "@tauri-apps/api/dialog";
import ToggleButton from "../Inputs/toggle-button";
const FileSettingComponent = ({ setter, status = "" }) => {
  return (
    <div
      className="whitespace-pre border-2 border-indigo-600 location-input px-5 py-2.5 rounded-md "
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
      <span className="flex-none font-semibold text-blue-500 shadow-sm">
        {status}
      </span>
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

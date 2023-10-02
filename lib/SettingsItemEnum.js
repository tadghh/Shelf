//call from rust to get dis
import { invoke } from "@tauri-apps/api/tauri";

export async function SettingsItems() {
  const initialData = await invoke("shelf_settings_values");
  const resultObject = {};

  for (const key in initialData) {
    resultObject[key] = initialData[key][0];
  }

  return resultObject;
}

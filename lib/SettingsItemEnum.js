//call from rust to get dis
import { invoke } from "@tauri-apps/api/tauri";

export async function SettingsItems() {

    return invoke("shelf_settings_values");
}
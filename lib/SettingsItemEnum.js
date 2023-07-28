//call from rust to get dis
import { invoke } from "@tauri-apps/api/tauri";

export async function SettingsItems() {
    return await invoke("shelf_settings_values");
}
import { SettingsTypes } from "./SettingsTypeEnum";
import {
  FileSettingComponent,
  ToggleSettingComponent,
} from "@/components/settings/setting-item-components";

export const getComponentForEnum = (enumValue) => {
  const componentMap = {
    [SettingsTypes.FILE]: FileSettingComponent,
    [SettingsTypes.TOGGLE]: ToggleSettingComponent,
  };

  return componentMap[enumValue] || null;
};

import { createProjectBridge } from "./projectBridge";
import { createSettingsBridge } from "./settingsBridge";
import { createShellBridge } from "./shellBridge";

export type {
  SaveQuickpanelBoundsInput,
  SaveUiSettingsInput,
} from "./settingsBridge";

export function createQuickPanelBridge() {
  return {
    ...createProjectBridge(),
    ...createSettingsBridge(),
    ...createShellBridge(),
  };
}

export type QuickPanelBridge = ReturnType<typeof createQuickPanelBridge>;

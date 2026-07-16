import type { GroupedView } from "../../lib/groupedView";
import type { ProjectState, QuickPanelMode, SortMode } from "../../models/types";

export type UpdateStatus = "idle" | "available" | "downloading" | "ready";

export type ShowInputPayload = {
  mode: string;
  title: string;
  value: string;
  closeOnSubmit?: boolean;
};

export type MountOptions = {
  focusDialogInput?: () => void;
};

export type QuickPanelState = {
  appState: ProjectState;
  quickName: string;
  newProjectName: string;
  commentText: string;
  alwaysOnTop: boolean;
  openOnStart: boolean;
  updateStatus: UpdateStatus;
  updateVersion: string;
  updateProgress: number;
  updatePromptOpen: boolean;
  quickPanelOpacity: number;
  sortMode: SortMode;
  quickPanelMode: QuickPanelMode;
  draggedProject: string | null;
  dropTargetProject: string | null;
  dropPosition: "before" | "after";
  contextMenuProject: string | null;
  contextMenuPosition: { x: number; y: number } | null;
  dialogOpen: boolean;
  dialogMode: string;
  aboutOpen: boolean;
  dialogTitle: string;
  dialogValue: string;
  closeInputOnSubmit: boolean;
};

export type QuickPanelView = {
  readonly minOpacity: number;
  readonly isCompactLayout: boolean;
  readonly effectiveSortMode: SortMode;
  readonly allProjects: string[];
  readonly projectColors: Record<string, string>;
  readonly projectGroups: Record<string, string>;
  readonly groupProjectsEnabled: boolean;
  readonly knownGroupNames: string[];
  readonly groupedProjects: GroupedView;
  readonly collapsedGroups: ReadonlySet<string>;
};

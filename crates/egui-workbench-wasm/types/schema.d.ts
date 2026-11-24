// Generated from the egui-workbench host JSON Schema. Do not edit by hand.



export interface CommandSpec {
  "category"?: string;
  "checked"?: boolean;
  "enabled"?: boolean;
  "id"?: string;
  "shortcut"?: string | null;
  "title"?: string;
  "when"?: string | null;
}

export interface DialogSpec {
  "cancel"?: string;
  "confirm"?: string;
  "id"?: string;
  "message"?: string;
  "open"?: boolean;
  "title"?: string;
}

export interface GroupSpec {
  "active"?: number;
  "id"?: string;
  "tabs"?: Array<TabSpec>;
}

export interface LayoutSpec {
  "active"?: number;
  "direction"?: string;
  "groups"?: Array<GroupSpec>;
}

export interface MenuItemSpec {
  "command"?: string | null;
  "label"?: string;
  "separator"?: boolean;
}

export interface MenuSpec {
  "items"?: Array<MenuItemSpec>;
  "title"?: string;
}

export interface PanelsSpec {
  "bottom"?: boolean;
  "inspector"?: boolean;
  "sidebar"?: boolean;
  "status"?: boolean;
}

export interface SeriesSpec {
  "color"?: string | null;
  "dataset"?: string | null;
  "name"?: string;
  "points"?: Array<Array<number>>;
  "style"?: string;
  "width"?: number | null;
  "x"?: string | null;
  "y"?: string | null;
}

export interface StatusSpec {
  "left"?: Array<string>;
  "right"?: Array<string>;
}

export interface TabSpec {
  "id"?: string;
  "pinned"?: boolean;
  "title"?: string;
}

export interface ThemeSpec {
  "follow_system"?: boolean;
  "id"?: string;
  "reduced_motion"?: boolean;
}

export interface ToastSpec {
  "description"?: string | null;
  "kind"?: string;
  "title"?: string;
}

export interface TreeSpec {
  "children"?: Array<TreeSpec>;
  "expanded"?: boolean;
  "icon"?: string | null;
  "key"?: string;
  "label"?: string;
}

export type WidgetSpec =
  | {
  "kind": "label";
  "size": number;
  "strong": boolean;
  "text": string;
  "tone": string;
}
  | {
  "enabled": boolean;
  "id": string;
  "kind": "button";
  "label": string;
  "variant": string;
}
  | {
  "hint": string;
  "id": string;
  "kind": "text_field";
  "label": string;
  "value": string;
}
  | {
  "checked": boolean;
  "id": string;
  "kind": "checkbox";
  "label": string;
}
  | {
  "id": string;
  "kind": "switch";
  "label": string;
  "on": boolean;
}
  | {
  "id": string;
  "kind": "select";
  "label": string;
  "options": Array<string>;
  "selected": number;
}
  | {
  "id": string;
  "kind": "slider";
  "label": string;
  "max": number;
  "min": number;
  "step": number;
  "value": number;
}
  | {
  "kind": "separator";
}
  | {
  "height": number;
  "kind": "spacer";
}
  | {
  "children": Array<WidgetSpec>;
  "gap": number;
  "kind": "row";
}
  | {
  "children": Array<WidgetSpec>;
  "gap": number;
  "kind": "column";
}
  | {
  "children": Array<WidgetSpec>;
  "kind": "card";
  "title": string;
}
  | {
  "active": number;
  "children": Array<Array<WidgetSpec>>;
  "id": string;
  "kind": "tabs";
  "tabs": Array<string>;
}
  | {
  "id": string;
  "kind": "tree";
  "nodes": Array<TreeSpec>;
  "selected"?: string | null;
}
  | {
  "dataset": string;
  "height": number;
  "id": string;
  "kind": "data_grid";
}
  | {
  "height": number;
  "id": string;
  "kind": "plot";
  "renderer": string;
  "series": Array<SeriesSpec>;
  "title": string;
}
  | {
  "kind": "custom";
  "props": unknown;
  "widget": string;
};

export interface WindowSpec {
  "custom_title_bar"?: boolean;
  "height"?: number;
  "resizable"?: boolean;
  "title"?: string;
  "width"?: number;
}

export interface HostSpec {
  "commands"?: Array<CommandSpec>;
  "dialogs"?: Array<DialogSpec>;
  "explorer"?: Array<TreeSpec>;
  "layout"?: LayoutSpec;
  "menus"?: Array<MenuSpec>;
  "metadata"?: {

};
  "panels"?: PanelsSpec;
  "status"?: StatusSpec;
  "theme"?: ThemeSpec;
  "toasts"?: Array<ToastSpec>;
  "version"?: number;
  "widgets"?: Array<WidgetSpec>;
  "window"?: WindowSpec;
}


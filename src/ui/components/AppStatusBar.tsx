import { Minus, X } from "lucide-react";

import { invoke } from "@tauri-apps/api/core";
import { window as viewWindow } from "@tauri-apps/api";

import "@css/components/AppStatusBar.scss";

function AppStatusBar() {
    return (
        <div
            id={"status-bar"}
            className={"flex h-fit w-full justify-end"}
            data-tauri-drag-region={true}
        >
            <div id={"status-bar__buttons"}>
                <div
                    id={"status-bar__minimize"}
                    className={"StatusBar__Button"}
                    onClick={() => viewWindow.getCurrentWindow().minimize()}
                >
                    <Minus color={"white"} strokeWidth={2.5} size={18} />
                </div>

                <div
                    id={"status-bar__close"}
                    className={"StatusBar__Button"}
                    onClick={() => invoke("window__close")}
                >
                    <X color={"white"} strokeWidth={2.5} size={18} />
                </div>
            </div>
        </div>
    );
}

export default AppStatusBar;

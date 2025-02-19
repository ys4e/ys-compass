import { Route, Routes } from "react-router-dom";

import useBackground from "@hooks/appearance/useBackground.ts";

import Home from "@pages/launcher/Home.tsx";

import AppStatusBar from "@components/AppStatusBar.tsx";

import "@css/Launcher.scss";
import VersionManager from "@pages/launcher/VersionManager.tsx";
import NavigationSideBar from "@components/launcher/NavigationSideBar.tsx";
import { SidebarProvider } from "@shad/sidebar.tsx";

/**
 * This is the main part of the application.
 * @constructor
 */
function Launcher() {
    const bgPath = useBackground();

    return (
        <SidebarProvider>
            <div
                id={"page__launcher"}
                style={{
                    background: bgPath ? `url(${bgPath})` : undefined,
                }}
            >
                <NavigationSideBar />

                <div className={"flex flex-col w-full"}>
                    <AppStatusBar />

                    <div id={"main-content"}>
                        <Routes>
                            <Route path={"/"} element={<Home />} />
                            <Route path={"/versions"} element={<VersionManager />} />
                        </Routes>
                    </div>
                </div>
            </div>
        </SidebarProvider>
    );
}

export default Launcher;

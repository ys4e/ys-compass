import { Route, Routes } from "react-router-dom";

import useBackground from "@hooks/appearance/useBackground.ts";

import Home from "@pages/launcher/Home.tsx";

import AppStatusBar from "@components/AppStatusBar.tsx";

import "@css/Launcher.scss";

/**
 * This is the main part of the application.
 * @constructor
 */
function Launcher() {
    const bgPath = useBackground();

    return (
        <div
            id={"page__launcher"}
            style={{
                background: bgPath ? `url(${bgPath})` : undefined,
            }}
        >
            <div id={"sidebar"}>

            </div>

            <div className={"flex flex-col w-full"}>
                <AppStatusBar />

                <div id={"main-content"}>
                    <Routes>
                        <Route path={"/"} element={<Home />} />
                    </Routes>
                </div>
            </div>
        </div>
    );
}

export default Launcher;

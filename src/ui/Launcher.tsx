import useBackground from "@hooks/appearance/useBackground.ts";

import AppStatusBar from "@components/AppStatusBar.tsx";

import "@css/App.scss";

import "react-contexify/ReactContexify.css";

/**
 * This is the main part of the application.
 * @constructor
 */
function Launcher() {
    const bgPath = useBackground();

    return (
        <div
            id={"app"}
            style={{
                background: bgPath ? `url(${bgPath})` : undefined,
            }}
        >
            <AppStatusBar />
        </div>
    );
}

export default Launcher;

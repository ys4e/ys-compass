import "react-contexify/ReactContexify.css";
import { createRoot } from "react-dom/client";
import { RouterProvider, createBrowserRouter } from "react-router-dom";

import posthog from "posthog-js";

import Launcher from "@ui/Launcher.tsx";
import PacketVisualizer from "@ui/PacketVisualizer.tsx";
import ReplayViewer from "@ui/ReplayViewer.tsx";

import "@css/global.scss";

export const router = createBrowserRouter([
    { path: "/*", element: <Launcher /> },
    { path: "/replay", element: <ReplayViewer /> },
    { path: "/visualizer", element: <PacketVisualizer /> }
]);

const root = createRoot(document.getElementById("root")!);
root.render(<RouterProvider router={router} />);

// Configure PostHog for analytics.
posthog.init(import.meta.env.VITE_POSTHOG_TOKEN, {
    api_host: "https://us.i.posthog.com",
    person_profiles: "never"
});

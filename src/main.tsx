import { createRoot } from "react-dom/client";
import { RouterProvider, createBrowserRouter } from "react-router-dom";

import posthog from "posthog-js";

import Launcher from "@ui/Launcher.tsx";
import ReplayViewer from "@ui/ReplayViewer.tsx";
import PacketVisualizer from "@ui/PacketVisualizer.tsx";

import "@css/global.scss";
import "react-contexify/ReactContexify.css";

export const router = createBrowserRouter([
    { path: "/*", element: <Launcher /> },
    { path: "/replay", element: <ReplayViewer /> },
    { path: "/visualizer", element: <PacketVisualizer /> }
]);

const root = createRoot(document.getElementById("root")!);
root.render(<RouterProvider router={router} />);

// Configure PostHog for analytics.
posthog.init(
    "phc_VVD9XbDn9QBsXWfhJvrYxxRuE0LScyXK3JZzJpD3E",
    { api_host: "https://us.i.posthog.com", person_profiles: "never" }
);

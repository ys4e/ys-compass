import { createRoot } from "react-dom/client";
import { RouterProvider, createBrowserRouter } from "react-router-dom";

import Launcher from "@ui/Launcher.tsx";

export const router = createBrowserRouter([{ path: "*", element: <Launcher /> }]);

const root = createRoot(document.getElementById("root")!);
root.render(<RouterProvider router={router} />);

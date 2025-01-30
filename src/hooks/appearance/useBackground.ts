import { useEffect, useState } from "react";

import Appearance from "@backend/Appearance.ts";

function useBackground() {
    const [path, setPath] = useState<string | undefined>(undefined);

    useEffect(() => {
        // Start a coroutine to fetch the background image.
        Appearance.getBackgroundImage().then(setPath);

        // On initialize, grab the first cached background image.
        (async () => {
            setPath(await Appearance.getBackgroundImage(true))
        })();
    }, []);

    return path;
}

export default useBackground;

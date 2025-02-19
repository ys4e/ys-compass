import { useEffect, useState } from "react";

import useBackground from "@hooks/appearance/useBackground.ts";

import Appearance from "@backend/Appearance.ts";
import { Colors } from "@backend/types.ts";

function useColorScheme(): Colors {
    const [palette, setPalette] = useState<Colors>({
        dark: true,
        primary: "#ffffff"
    });

    const backgroundPath = useBackground();

    useEffect(() => {
        Appearance.getColorScheme(undefined).then(setPalette);
    }, []);

    useEffect(() => {
        if (backgroundPath == undefined) return;

        Appearance.getColorScheme(backgroundPath).then(setPalette);
    }, [backgroundPath]);

    return palette;
}

export default useColorScheme;

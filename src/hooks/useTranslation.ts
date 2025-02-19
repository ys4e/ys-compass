import { UnlistenFn, listen } from "@tauri-apps/api/event";
import { warn } from "@tauri-apps/plugin-log";
import { useEffect, useState } from "react";

import Language, { TranslateArguments } from "@backend/Language.ts";

/**
 * Translates the given key to the current language.
 * This is a React hook that listens for language changes.
 *
 * @param key The key to translate.
 * @param args The arguments to replace in the text.
 */
function useTranslation(
    key: string,
    args?: TranslateArguments | undefined
): string {
    const [value, setValue] = useState<string | undefined>(undefined);

    useEffect(() => {
        Language.translate(key, args).then(setValue);
    }, []);

    useEffect(() => {
        let unlisten: UnlistenFn = () => warn("unlisten not set");

        // Listen for language changes.
        listen("ysc://language/change", () => {
            Language.translate(key, args).then(setValue);
        }).then((fn) => (unlisten = fn));

        return () => unlisten();
    }, []);

    return value ?? "";
}

export default useTranslation;

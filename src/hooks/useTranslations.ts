import { UnlistenFn, listen } from "@tauri-apps/api/event";
import { warn } from "@tauri-apps/plugin-log";
import { useEffect, useState } from "react";

import Global from "@backend/Global.ts";
import Language from "@backend/Language.ts";

type Translate = (key: string) => string;
type Translations = Map<string, string>;

/**
 * Allows translating multiple keys to the current language.
 * This is a React hook that listens for language changes.
 */
function useTranslations(): Translate {
    const [map, setMap] = useState<Translations>(new Map());

    /**
     * Fetches the translation for the given key.
     *
     * @param key The key to translate.
     */
    async function fetchTranslation(key: string) {
        Language.translate(key).then((string) => {
            setMap((prev) => {
                prev.set(key, string);
                return prev;
            });
        });
    }

    useEffect(() => {
        let unlisten: UnlistenFn = () => warn("unlisten not set");

        // Listen for language changes.
        listen("ysc://language/change", () => {
            // Re-fetch all translations.
            for (const key of map.keys()) {
                fetchTranslation(key).catch(Global.fallback);
            }
        }).then((fn) => (unlisten = fn));

        return () => unlisten();
    }, []);

    return (key: string) => {
        const string = map.get(key);

        // Look up the string if it hasn't been fetched yet.
        if (string == undefined) {
            fetchTranslation(key).catch(Global.fallback);
            return "";
        }

        return string;
    };
}

export default useTranslations;

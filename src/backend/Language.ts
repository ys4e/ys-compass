import { invoke } from "@tauri-apps/api/core";

export type TranslateArguments = { [key: string]: string };

/**
 * Translates the given key to the current language.
 * This is a shortcut method for the underlying Language class.
 *
 * @param key The key to translate.
 * @param args The arguments to replace in the text.
 */
export async function t(
    key: string,
    args?: TranslateArguments
): Promise<string> {
    return Language.translate(key, args);
}

class Language {
    /**
     * Translates the given text to the current language.
     *
     * @param text The text to translate.
     * @param args The arguments to replace in the text.
     */
    public static async translate(
        text: string,
        args?: TranslateArguments | undefined
    ): Promise<string> {
        return await invoke("translate", { key: text, args });
    }
}

export default Language;

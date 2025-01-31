import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { error as logError } from "@tauri-apps/plugin-log";

import Global from "@backend/Global.ts";

class Appearance {
    /**
     * Returns a browser-safe URL to the background image.
     *
     * This function is for the initial loading of the background image.
     * After the initial load, the application backend will query the API for updates.
     *
     * This falls back to the default background image if one could not be resolved.
     *
     * @param cache Should the cache be strictly used?
     */
    public static async getBackgroundImage(cache: boolean = false): Promise<string> {
        let backgroundPath: string | undefined = undefined;

        if (cache) {
            // Get the cached background from the store.
            backgroundPath = await Appearance.getLastBackground();
        } else try {
            // Use the latest background from the backend.
            backgroundPath = await invoke("appearance__background") as string;

            // If this doesn't fail, we should set this value in the cache.
            await Global.getCacheStore().set("last-background", backgroundPath);
        } catch (error) {
            // Fallback to the default background.
            // This will just leave backgroundPath as undefined.
        }

        // If the background is still undefined, fallback to the default background.
        if (backgroundPath == undefined) try {
            // Query the backend for the background image.
            backgroundPath = await invoke("appearance__default_splash") as string;
        } catch (error) {
            // This is a CRITICAL error.
            logError("Failed to get the default background image.")
                .catch(error => Global.fallback(error));

            return "";
        }

        return convertFileSrc(backgroundPath);
    }

    /**
     * Returns the path to the last background image.
     * Returns undefined if no background image was found.
     *
     * @private
     */
    private static async getLastBackground(): Promise<string | undefined> {
        return await Global.getCacheStore().get<string>("last-background");
    }
}

export default Appearance;

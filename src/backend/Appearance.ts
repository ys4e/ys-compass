import { convertFileSrc, invoke } from "@tauri-apps/api/core";

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
        if (cache) {
            // TODO: Get last displayed background.
            // This should probably be stored in a SQLite database.

            const cachedPath = await invoke("appearance__default_splash") as string;
            return convertFileSrc(cachedPath);
        }

        try {
            // Query the backend for the background image.
            const backgroundImage = await invoke("appearance__background") as string;
            return convertFileSrc(backgroundImage);
        } catch (error) {
            // TODO: Log the error.
            return "";
        }
    }
}

export default Appearance;

import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { error as logError } from "@tauri-apps/plugin-log";

import Global from "@backend/Global.ts";
import { Colors } from "@backend/types.ts";

import tinycolor from "tinycolor2";
import { FastAverageColor } from "fast-average-color";

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
     * Sets the theme of the application.
     *
     * @param theme The theme to set the application to.
     */
    public static setTheme(theme: "dark" | "light"): void {
        document
            .getElementsByTagName("html")[0]
            .setAttribute("data-theme", theme);
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

    /**
     * Calculates the average color of the background.
     * This uses `fast-average-color`.
     *
     * The result is an object of different HEX colors to use.
     */
    public static async getColorScheme(path?: string | undefined): Promise<Colors> {
        // Get the background image if it wasn't provided.
        path ??= await Appearance.getBackgroundImage(true);

        // Calculate the color of the image.
        const calculator = new FastAverageColor();
        const result = await calculator.getColorAsync(path);

        // Saturate the color.
        const wrapper = tinycolor(result.hex)
            .saturate(20)
            .lighten(35);

        return {
            dark: result.isDark,
            primary: wrapper.toHexString(),
        };
    }
}

export default Appearance;

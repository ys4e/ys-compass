import { Store } from "@tauri-apps/plugin-store";
import { warn as logWarn } from "@tauri-apps/plugin-log";

const cacheStore = await Store.load("cache.json");

class Global {
    /**
     * Global accessor for the cache store.
     */
    static getCacheStore(): Store {
        return cacheStore;
    }

    /**
     * This method is a LAST RESORT for error catching.
     */
    static fallback(error: any): void {
        console.error("Crashed while crashing. Please report this bug to the developers!", error);
    }

    /**
     * Logs a warning message to the console.
     *
     * @param message The message to log.
     */
    static warn(message: string): void {
        logWarn(message).catch(Global.fallback);
    }
}

export default Global;

import { Store } from "@tauri-apps/plugin-store";

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
}

export default Global;

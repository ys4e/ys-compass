import { invoke } from "@tauri-apps/api/core";

class Capabilities {
    /**
     * Opens a window for the packet sniffer.
     */
    public static async openVisualizer(): Promise<void> {
        await invoke("sniffer__run");
        await invoke("sniffer__open");
    }
}

export default Capabilities;

export type ServerAddress = `${string}:${number}`;

/**
 * JSON-serialized packet information.
 */
export type Packet = {
    time: number;
    source: "client" | "server";
    packetId: number;
    packetName: string;
    length: number;
    data: string; // This is a JSON string.

    index?: number; // A client-side index for ordering packets.
    binary?: string; // Base64-encoded raw packet data.
};

/**
 * JSON-serialized version information.
 */
export type Version = {
    version: string;
    path: string;
};

/**
 * JSON-serialized mod information.
 */
export type Tool = {
    id: string;
    name: string;
    icon: string;
    path: string;
};

/**
 * JSON-serialized mod information.
 */
export type Mod = {
    id: string;
    name: string;
    icon: string;
    path: string;
    version: string;
    tool: Tool;
};

/**
 * JSON-serialized version of a launch profile.
 */
export type Profile = {
    id: string;
    name: string;
    icon: string;
    version: Version;
    tools: Tool[];
    mods: Mod[];
    launch_args: string;
};

/**
 * Color palette object.
 */
export type Colors = {
    dark: boolean;
    primary: string;
};

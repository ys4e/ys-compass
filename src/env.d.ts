/// <reference types="vite/client" />

interface ImportMetaEnv {
    readonly MODE: string;
    readonly VITE_POSTHOG_TOKEN: string;

    // more env variables...
}

interface ImportMeta {
    readonly env: ImportMetaEnv;
}

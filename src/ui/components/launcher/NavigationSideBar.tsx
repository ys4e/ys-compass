import {
    AppWindow,
    BookDown,
    Hammer,
    Home,
    Menu,
    Monitor,
    NotepadText,
    Package,
    Settings
} from "lucide-react";

import {
    Sidebar,
    SidebarContent,
    SidebarGroup,
    SidebarGroupLabel,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
    useSidebar
} from "@shad/sidebar.tsx";
import { info } from "@tauri-apps/plugin-log";
import { ExoticComponent, useEffect, useRef } from "react";

import Button from "@components/common/Button.tsx";
import Text from "@components/common/Text.tsx";

import useTranslations from "@hooks/useTranslations.ts";

import { cn } from "@app/utils.ts";

type Item = {
    /**
     * This needs to be a locale string.
     */
    label: string;

    /**
     * The icon to display.
     */
    icon: ExoticComponent;

    /**
     * Click event handler.
     */
    onClick?: () => void;

    /**
     * The path to navigate to.
     */
    navigate?: string;

    /**
     * Whether the item should be bold.
     */
    bold?: boolean | undefined;

    /**
     * Should this button trigger the menu?
     */
    menu?: boolean | undefined;
};

const isolated = [
    {
        label: "launcher.sidebar.menu",
        icon: Menu,
        onClick: () => info("test"),
        bold: true,
        menu: true
    },
    {
        label: "launcher.sidebar.launcher",
        icon: Home,
        navigate: "/"
    }
] as Item[];

const game = [
    {
        label: "launcher.sidebar.game.versions",
        icon: BookDown,
        onClick: () => info("version manager")
    },
    {
        label: "launcher.sidebar.game.tools",
        icon: Hammer,
        onClick: () => info("tool manager")
    },
    {
        label: "launcher.sidebar.game.mods",
        icon: Package,
        onClick: () => info("mod browser")
    }
] as Item[];

const utilities = [
    {
        label: "launcher.sidebar.utilities.visualizer",
        icon: Monitor,
        onClick: Capabilities.openVisualizer
    },
    {
        label: "launcher.sidebar.utilities.editor",
        icon: NotepadText,
        onClick: () => info("packet editor")
    }
] as Item[];

const settings = [
    {
        label: "launcher.sidebar.profiles",
        icon: AppWindow,
        onClick: () => info("profiles")
    },
    {
        label: "launcher.sidebar.settings",
        icon: Settings,
        onClick: () => info("settings")
    }
] as Item[];

/**
 * Renders a sidebar menu.
 *
 * @param array The items to render.
 */
function sidebarMenu(array: Item[]) {
    const navigate = useNavigate();
    const { toggleSidebar } = useSidebar();

    return (
        <SidebarMenu>
            {array.map((item) => (
                <SidebarMenuItem key={item.label}>
                    <SidebarMenuButton asChild={true}>
                        <Button
                            onClick={() => {
                                if (item.menu) {
                                    toggleSidebar()
                                } else if (item.onClick) {
                                    item.onClick();
                                } else if (item.navigate) {
                                    navigate(item.navigate);
                                }
                            }}
                            className={"flex flex-row"}
                        >
                            <item.icon />

                            <Text
                                className={cn(
                                    "select-none",
                                    "group-data-[collapsible=icon]:hidden",
                                    item.bold && "font-semibold"
                                )}
                                theme={"secondary"}
                            >
                                {item.label}
                            </Text>
                        </Button>
                    </SidebarMenuButton>
                </SidebarMenuItem>
            ))}
        </SidebarMenu>
    );
}

function NavigationSideBar() {
    const sidebar = useSidebar();
    const t = useTranslations();

    const ref = useRef<HTMLDivElement>(null);

    /**
     * Detect clicks outside the sidebar.
     */
    useEffect(() => {
        const handler = (event: MouseEvent) => {
            if (!ref.current) {
                return;
            }

            if (!ref.current.contains(event.target as Node)) {
                sidebar.setOpen(false);
            }
        };

        window.addEventListener("click", handler);
        return () => window.removeEventListener("click", handler);
    }, []);

    return (
        <div ref={ref} className={"absolute"}>
            <Sidebar
                id={"sidebar"}
                collapsible={"icon"}
                className={cn(
                    "rounded-r-lg overflow-hidden",
                    "backdrop-blur-lg group-data-[collapsible=icon]:backdrop-blur-none",
                    "backdrop-saturate-120 group-data-[collapsible=icon]:backdrop-saturate-100",
                    "backdrop-brightness-80 group-data-[collapsible=icon]:backdrop-brightness-100"
                )}
            >
                <SidebarContent
                    className={"h-screen flex flex-col justify-between"}
                >
                    <div>
                        <SidebarGroup>{sidebarMenu(isolated)}</SidebarGroup>

                        <SidebarGroup>
                            <SidebarGroupLabel>
                                {t("launcher.sidebar.game")}
                            </SidebarGroupLabel>

                            <SidebarMenu>{sidebarMenu(game)}</SidebarMenu>
                        </SidebarGroup>

                        <SidebarGroup>
                            <SidebarGroupLabel>
                                {t("launcher.sidebar.utilities")}
                            </SidebarGroupLabel>

                            <SidebarMenu>{sidebarMenu(utilities)}</SidebarMenu>
                        </SidebarGroup>
                    </div>

                    <SidebarGroup>{sidebarMenu(settings)}</SidebarGroup>
                </SidebarContent>
            </Sidebar>
        </div>
    );
}

export default NavigationSideBar;

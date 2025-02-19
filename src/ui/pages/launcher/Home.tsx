import Button from "@components/common/Button.tsx";
import useColorScheme from "@hooks/appearance/useColorScheme.ts";
import classNames from "classnames";
import { ChevronsRight } from "lucide-react";
import Text from "@components/common/Text.tsx";
import { invoke } from "@tauri-apps/api/core";
import { t } from "@backend/Language.ts";

function Home() {
    const colors = useColorScheme();

    return (
        <div
            id={"page__home"}
            className={"w-full h-full flex flex-row justify-between"}
        >
            <div
                className={"h-full flex flex-col justify-end ml-12 pl-8 py-9"}
            >
                <Button
                    className={classNames(
                        "lifted w-[340px] h-10 px-5",
                        "flex flex-row items-center justify-between",
                        "text-sm text-left font-medium text-primary",
                        "rounded-full",
                        "transition-transform duration-200",
                        "hover:scale-105 hover:opacity-85",
                        "active:opacity-75"
                    )}
                    style={{
                        backgroundColor: colors.primary,
                    }}
                    onClick={async () => {
                        try {
                            await invoke("game__launch");
                        } catch (error) {
                            console.error(await t(error as string));
                        }
                    }}
                >
                    <Text>launcher.home.game.play</Text>

                    <ChevronsRight size={22} />
                </Button>
            </div>

            <div>

            </div>
        </div>
    );
}

export default Home;

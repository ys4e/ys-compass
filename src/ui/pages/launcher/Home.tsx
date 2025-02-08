import Button from "@components/common/Button.tsx";
import useColorScheme from "@hooks/appearance/useColorScheme.ts";
import classNames from "classnames";
import { RefreshCw } from "lucide-react";

function Home() {
    const colors = useColorScheme();

    return (
        <div
            id={"page__home"}
            className={"w-full h-full flex flex-row justify-between"}
        >
            <div
                className={"h-full flex flex-col justify-end pl-8 py-9"}
            >
                <Button
                    className={classNames(
                        "lifted w-[340px] h-10 px-5",
                        "flex flex-row items-center justify-between",
                        "text-sm text-left font-medium text-primary",
                        "rounded-full"
                    )}
                    style={{
                        backgroundColor: colors.primary,
                    }}
                    onClick={() => {
                        console.log("Button clicked!");
                    }}
                >
                    {"Update Game"}

                    <RefreshCw size={22} />
                </Button>
            </div>

            <div>

            </div>
        </div>
    );
}

export default Home;

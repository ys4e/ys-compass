import Button from "@components/common/Button.tsx";
import useColorScheme from "@hooks/appearance/useColorScheme.ts";

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
                    className={"lifted w-[340px] h-10 px-4 text-md text-left text-white-100 rounded-full"}
                    style={{
                        backgroundColor: colors.primary,
                    }}
                    onClick={() => {
                        console.log("Button clicked!");
                    }}
                >
                    Hello world!
                </Button>
            </div>

            <div>

            </div>
        </div>
    );
}

export default Home;

import { Button, MantineProvider } from "@mantine/core";
import { execute } from "./api/base";
import { useTerminals } from "./api/hooks";

export function App() {
    const terminals = useTerminals();
    console.log(terminals);

    return (
        <MantineProvider>
            <Button
                onClick={() =>
                    execute("create_terminal", {
                        command: "zsh",
                        args: null,
                        title: null,
                    }).then(console.log)
                }
            >
                New Terminal
            </Button>
            <Button onClick={() => execute("get_terminals").then(console.log)}>
                List Terminals
            </Button>
        </MantineProvider>
    );
}

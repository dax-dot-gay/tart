import { Button, MantineProvider } from "@mantine/core";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect } from "react";

export function App() {
    useEffect(() => {
        let unlisten: UnlistenFn = () => {};
        listen<any>("tart://event", console.log).then((v) => (unlisten = v));
        return unlisten;
    }, []);

    return (
        <MantineProvider>
            <Button
                onClick={() =>
                    invoke("execute_command", {
                        command: {
                            type: "CreateTerminal",
                            command: "zsh",
                            args: null,
                            title: null,
                        },
                    }).then(console.log)
                }
            >
                New Terminal
            </Button>
            <Button
                onClick={() =>
                    invoke("execute_command", {
                        command: {
                            type: "GetTerminals",
                        },
                    }).then(console.log)
                }
            >
                List Terminals
            </Button>
        </MantineProvider>
    );
}

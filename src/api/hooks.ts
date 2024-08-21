import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { omit } from "lodash";
import { useCallback, useEffect, useState } from "react";
import { Terminal } from "./types";
import { execute } from "./base";

export function useAppEvent<T extends object = {}>(
    type: string,
    callback: (data: T) => void
) {
    useEffect(() => {
        let unlisten: UnlistenFn = () => {};
        listen<{ type: string } & T>("tart://event", (evt) => {
            if (evt.payload.type === type) {
                callback(omit<T>(evt.payload, "type") as any);
            }
        }).then((f) => (unlisten = f));
        return unlisten;
    }, [type, callback]);
}

export function useTerminals(): Terminal[] {
    const [terminals, setTerminals] = useState<Terminal[]>([]);
    const onEvent = useCallback(
        () =>
            execute<Terminal[]>("get_terminals").then((result) => {
                if (result.success) {
                    setTerminals(result.data);
                } else {
                    setTerminals([]);
                }
            }),
        [setTerminals]
    );

    useAppEvent("TerminalCreated", onEvent);
    useAppEvent("TerminalResized", onEvent);
    useAppEvent("TerminalRemoved", onEvent);

    useEffect(() => {
        onEvent();
    }, [onEvent]);

    return terminals;
}

import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { SearchAddon } from "@xterm/addon-search";
import { WebglAddon } from "@xterm/addon-webgl";
import { SerializeAddon } from "@xterm/addon-serialize";
import { PtySize, Terminal } from "../api/types";
import { useCallback, useEffect, useRef, useState } from "react";
import { useAppEvent } from "../api/hooks";
import { execute } from "../api/base";

type TerminalContext = {
    terminal: XTerm;
    addons: {
        fit: FitAddon;
        search: SearchAddon;
        webgl: WebglAddon;
        serialize: SerializeAddon;
    };
};

type TerminalState = {
    size: {
        rows: number;
        cols: number;
    };
    content: string;
};

export function TerminalPanel({ terminal }: { terminal: Terminal }) {
    const createTerminal = useCallback(
        (
            size: Omit<PtySize, "pixel_height" | "pixel_width">,
            content: string
        ) => {
            const fit = new FitAddon();
            const search = new SearchAddon();
            const webgl = new WebglAddon();
            const serialize = new SerializeAddon();

            const terminal = new XTerm({
                allowTransparency: true,
                rows: size.rows,
                cols: size.cols,
                disableStdin: false,
            });
            terminal.loadAddon(fit);
            terminal.loadAddon(search);
            terminal.loadAddon(webgl);
            terminal.loadAddon(serialize);
            terminal.write(content);

            return {
                terminal,
                addons: {
                    fit,
                    search,
                    webgl,
                    serialize,
                },
            };
        },
        []
    );

    const terminalRef = useRef<HTMLDivElement | null>(null);
    const [term, setTerm] = useState<TerminalContext>(
        createTerminal(terminal.size, "")
    );
    const [terminalState, setTerminalState] = useState<TerminalState>({
        size: { rows: term.terminal.rows, cols: term.terminal.cols },
        content: "",
    });

    useEffect(() => {
        if (terminalRef.current) {
            term.terminal.open(terminalRef.current);
        }

        return () => term.terminal.dispose();
    }, [terminalRef.current, term.terminal]);

    useEffect(() => {
        setTerminalState({
            size: { rows: term.terminal.rows, cols: term.terminal.cols },
            content: term.addons.serialize.serialize(),
        });
        setTerm(createTerminal(terminalState.size, terminalState.content));
    }, [terminal.id, setTerm, setTerminalState]);

    const onRead = useCallback(
        ({ id, data }: { id: string; data: string }) => {
            if (id === terminal.id) {
                term.terminal.write(data);
            }
        },
        [term.terminal, terminal.id]
    );

    useEffect(() => {
        const dispose = term.terminal.onData((data: string) => {
            term.addons.fit.fit();
            execute("write_data", { id: terminal.id, data });
        });
        return dispose.dispose;
    }, [terminal.id, term.terminal]);

    term.addons.fit.fit();
    console.log(term.addons.fit.proposeDimensions());

    useEffect(() => {
        console.log(term.addons.fit.proposeDimensions());
        term.addons.fit.fit();
    });

    useAppEvent<{ id: string; data: string }>("TerminalRead", onRead);

    return <div className="terminal-container" ref={terminalRef}></div>;
}

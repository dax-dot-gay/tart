import { Box } from "@mantine/core";
import { Terminal } from "../api/types";
import { TerminalPanel } from "../components/term";

export function TerminalView({
    terminal,
    visible,
}: {
    terminal: Terminal;
    visible: boolean;
}) {
    return (
        <Box
            className="terminal-tab"
            display={visible ? "inline-block" : "none"}
        >
            <TerminalPanel terminal={terminal} />
        </Box>
    );
}

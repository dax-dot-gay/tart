import {
    ActionIcon,
    Box,
    Button,
    Divider,
    Group,
    Stack,
    Text,
} from "@mantine/core";
import { IconCookieFilled, IconPlus, IconTerminal } from "@tabler/icons-react";
import { useTerminals } from "../api/hooks";
import { useState } from "react";
import { Terminal } from "../api/types";
import { execute } from "../api/base";
import { TerminalView } from "./Terminal";

export function Layout() {
    const terminals = useTerminals();
    const [current, setCurrent] = useState<Terminal | null>(null);

    return (
        <Stack className="layout" gap={0}>
            <Group className="nav" gap="xs" px="xs" wrap="nowrap">
                <IconCookieFilled size={28} />
                <Divider orientation="vertical" />
                <Group gap="xs" className="tabs" wrap="nowrap">
                    {terminals.map((terminal) => (
                        <Button
                            key={terminal.id}
                            onClick={() => setCurrent(terminal)}
                            variant={
                                terminal.id === current?.id ? "filled" : "light"
                            }
                            leftSection={<IconTerminal size={20} />}
                        >
                            {terminal.title ?? terminal.command}
                        </Button>
                    ))}
                </Group>
                <ActionIcon
                    size="md"
                    variant="subtle"
                    onClick={() =>
                        execute("create_terminal", {
                            command: "zsh",
                            args: null,
                            title: "Shell",
                        })
                    }
                >
                    <IconPlus size={20} />
                </ActionIcon>
            </Group>
            <Divider />
            <Box className="content">
                {terminals.map((terminal) => (
                    <TerminalView
                        visible={current?.id === terminal.id}
                        terminal={terminal}
                        key={terminal.id}
                    />
                ))}
            </Box>
        </Stack>
    );
}

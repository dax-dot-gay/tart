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

export function Layout() {
    const terminals = useTerminals();
    const [current, setCurrent] = useState<Terminal | null>(null);

    return (
        <Stack className="layout" gap={0}>
            <Group className="nav" gap="sm" px="xs" wrap="nowrap">
                <IconCookieFilled size={28} />
                <Divider orientation="vertical" />
                <Group gap="xs" className="tabs" wrap="nowrap">
                    {terminals.map((terminal) => (
                        <Button
                            key={terminal.id}
                            variant={
                                terminal.id === current?.id ? "light" : "subtle"
                            }
                            leftSection={<IconTerminal size={20} />}
                        >
                            {terminal.title ?? terminal.command}
                        </Button>
                    ))}
                </Group>
                <ActionIcon size="md" variant="subtle">
                    <IconPlus size={20} />
                </ActionIcon>
            </Group>
            <Divider />
            <Box className="content"></Box>
        </Stack>
    );
}

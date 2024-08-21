import { invoke } from "@tauri-apps/api/core";
import { camelCase, upperFirst } from "lodash";

export type Command<TCommand extends string, TData extends object = {}> = {
    type: TCommand;
} & TData;

export type CommandResult<
    TCommand extends Command<any, any> = any,
    TData = any,
    TError = any
> = {
    id: string;
    command: TCommand;
} & ({ success: false; data: TError } | { success: true; data: TData });

type BareCommandResult<
    TCommand extends Command<any, any> = any,
    TData = any,
    TError = any
> = {
    id: string;
    command: TCommand;
    result: { Ok?: TData; Err?: TError };
};

export async function execute<
    TResult = any,
    TError = any,
    TCommand extends string = any,
    TData extends object = any
>(
    command: TCommand,
    data?: TData
): Promise<CommandResult<Command<TCommand, TData>, TResult, TError>> {
    const cmd = {
        type: upperFirst(camelCase(command)),
        ...(data ?? {}),
    };
    const result = await invoke<
        BareCommandResult<Command<TCommand, TData>, TResult, TError>
    >("execute_command", { command: cmd });
    if (result.result.Ok !== undefined) {
        return {
            id: result.id,
            command: result.command,
            success: true,
            data: result.result.Ok,
        };
    } else if (result.result.Err !== undefined) {
        return {
            id: result.id,
            command: result.command,
            success: false,
            data: result.result.Err,
        };
    } else {
        return {
            id: result.id,
            command: result.command,
            success: false,
            data: null as any,
        };
    }
}

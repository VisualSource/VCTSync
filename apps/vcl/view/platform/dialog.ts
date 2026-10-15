/** 
 * A mimic of tauri's dialog js package since they also use the some underlining dialog crate
 * @link https://v2.tauri.app/reference/javascript/dialog
*/
import { invoke } from "react-iced-native";


type OpenDialogOptions = {
    defaultPath?: string;
    directory?: boolean;
    multiple?: boolean;
    title?: string;
}

export const open = (opts: OpenDialogOptions = {}) => {
    return invoke<{ path: string | null; }>("open-folder", opts)
}

type ConfirmDialogOptions = {
    cancelLabel?: string;
    kind?: "info" | "warning" | "error",
    okLabel?: string;
    title?: string;
}

export const confirm = async (msg: string, options?: ConfirmDialogOptions) => {
    const result = await message(msg, {
        buttons: "OkCancel",
        title: options?.title ?? "Confirm",
        ...options
    })

    return result === "Ok";
}


type MessageDialogOptions = {
    title: string;
    buttons: "Ok" | "OkCancel" | "YesNo" | "YesNoCancel" | string & {}
    kind?: "info" | "warning" | "error";
}

export const message = async (message: string, options?: Omit<MessageDialogOptions, "description">) => {
    const data = await invoke<{ result: string }>("message-dialog", { ...options, description: message });

    return data.result;
}
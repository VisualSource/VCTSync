import { invoke } from "react-iced-native"

type DialogOptions = {
    title: string;
    description: string;
    buttons: "Ok" | "OkCancel" | "YesNo" | "YesNoCancel" | string & {}
    level?: "info" | "warning" | "error"
}

export const confirmDialog = (opts: DialogOptions) => {
    return invoke<{ result: string }>("message-dialog", opts);
}

type OpenOptions = {
    start?: string;
    title?: string;

}

export const openDialog = (opts: OpenOptions = {}) => {
    return invoke<{ path: string | null; }>("open-folder", opts)
}
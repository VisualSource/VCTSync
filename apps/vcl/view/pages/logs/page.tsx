import { useQuery } from "@tanstack/react-query";
import { invoke } from "react-iced-native";
import { useApp } from "../../state.js";
import { LogViewer } from "./log-viewer.js";

export const Logs = () => {
    const modLogLines = useApp(state => state.modOnlyLogLines);
    const setModLogLines = useApp(state => state.setModOnlyLogLines);
    const { isLoading, isError, error, data } = useQuery({
        queryKey: ["logs", modLogLines],
        queryFn: () => {
            return invoke<string[]>("logs::list-lines", {});
        }
    });

    return (
        <col>
            <row padding={[2, 4]}>
                <input label="Mod lines only" type="checkbox" checked={modLogLines} onChange={(e) => setModLogLines(e.value)} />
                <space width="fill" />
            </row>
            <hr />
            <LogViewer isError={isError} isLoading={isLoading} error={error} data={data} />
        </col>
    );
}
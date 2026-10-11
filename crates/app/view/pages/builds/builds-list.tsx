import { invoke } from "react-iced-native";
import HardDriveDownload from "../../assets/icons/hard-drive-download.svg" with { type: "svg" };
import type { Version } from "./types.js";
import { Tooltip } from "../../utils.js";


type Props = { icon: SvgHandle, isLoading: boolean; isError: boolean; error: Error | null, data?: Version[] | undefined }

export const BuildsList = ({
    isLoading, isError, error, data, icon }: Props) => {
    if (isLoading) return <col><text>Loading</text></col>

    if (isError) return (
        <col>
            <text>{error?.message ?? String(error)}</text>
        </col>
    );


    if (!data || Array.isArray(data) && data.length === 0) {
        return (
            <col>
                <text>No versions</text>
            </col>
        );
    }

    return (
        <col>
            {data.map(e => {
                return (
                    <row key={e.id} padding={[4, 8]} alignY="center" width="fill">
                        <view width="shrink">
                            <svg src={icon} width={24} height={24} />
                        </view>
                        <space width={10} />
                        <view style="roundedBox" padding={[4, 8]}>
                            <text center>
                                {e.version}
                            </text>
                        </view>
                        <space width="fill" />
                        <row spacing={4} alignY="center">
                            {e.branch !== "main" ? (
                                <view>
                                    <text center>{e.branch}</text>
                                </view>
                            ) : null}

                            <view>
                                <text center>{e.timestamp}</text>
                            </view>
                            <Tooltip tip="Install" position="left">
                                <button onPress={() => {
                                    invoke("install::build", e).catch(e => console.error(e));
                                }}>
                                    <svg width={24} height={24} src={HardDriveDownload} />
                                </button>
                            </Tooltip>
                        </row>
                    </row>
                );
            })}
        </col>
    );
}
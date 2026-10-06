import { queryOptions, useQueries, useQuery } from "@tanstack/react-query";
import { invoke } from "react-iced-native";

import Network from "../assets/icons/network.svg" with { type: "svg" };
import HardDriveDownload from "../assets/icons/hard-drive-download.svg" with { type: "svg" };
import FlaskConical from "../assets/icons/flask-conical.svg" with { type: "svg" };
import { Tooltip } from "../utils.js";

type GithubVersion = {
    tag_name: string;
    name?: string;
    prerelease: boolean;
    draft: boolean;
    target_commitish: string;
    published_at: string;
    assets: { node_id: string; name: string; browser_download_url: string; size: number; digest: string, content_type: string; }[];
}

type Version = {
    id: string;
    version: string;
    source_url: string;
    content_type: "local" | "remote",
    timestamp: string;
    branch: "master",
    digest: string;
}

export const Builds = () => {
    const [remote, local] = useQueries({
        queries: [
            queryOptions({
                queryKey: ["builds::remote"],
                queryFn: async () => {
                    let response = await fetch("https://api.github.com/repos/VisualSource/VoidCrewTerminus/releases", {
                        headers: {
                            "Accept": "application/vnd.github+json"
                        }
                    });

                    if (!response.ok) throw new Error(`failed to load remote versions: ${response.statusText}`);

                    const request = await response.json() as GithubVersion[];

                    return request.map(e => ({
                        id: e.assets[0]?.node_id,
                        version: e.tag_name,
                        timestamp: e.published_at,
                        content_type: "remote",
                        source_url: e.assets[0]?.browser_download_url,
                        digest: e.assets[0]?.digest,
                        size: e.assets[0]?.size,
                        branch: e.target_commitish
                    } as Version));
                }
            }),
            queryOptions({
                queryKey: ["builds::local"],
                queryFn: async () => {
                    // request

                    return [] as Version[]
                }
            })
        ],
    });


    return (
        <col>
            <view>
                <text size={24} >Versions</text>
            </view>
            <InstalledVersion />

            <space height={15} />

            <col spacing={4}>
                <row padding={2} alignY="center">
                    <text>Remote</text>
                    <space width="fill" />
                    <button disabled={remote.isLoading} onPress={() => void remote.refetch().catch(err => console.error(err))}>
                        Refetch
                    </button>
                </row>
                <hr />

                <scroll spacing={4}>
                    <BuildsList
                        isError={remote.isError}
                        error={remote.error}
                        isLoading={remote.isLoading}
                        data={remote.data}
                        icon={Network} />
                </scroll>

            </col>

            <space height={15} />

            <col>
                <row>
                    <text>Local</text>
                    <space width="fill" />
                    <button disabled={local.isLoading} onPress={() => void local.refetch().catch(e => console.error(e))}>
                        Refetch
                    </button>
                </row>
                <hr />

                <scroll>
                    <BuildsList isError={local.isError} error={local.error} isLoading={local.isLoading} data={local.data} icon={FlaskConical} />
                </scroll>

            </col>
        </col>
    );
}

const BuildsList = ({ isLoading, isError, error, data, icon }: { icon: SvgHandle, isLoading: boolean; isError: boolean; error: Error | null, data?: Version[] | undefined }) => {
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
        <>
            {data.map(e => {
                return (
                    <row key={e.id} padding={[4, 8]} alignY="center">
                        <view width="shrink">
                            <svg src={icon} width={24} height={24} />
                        </view>
                        <space width={10} />
                        <view style="roundedBox" padding={[4, 8]}>
                            <text>
                                {e.version}
                            </text>
                        </view>
                        <space width="fill" />
                        <row spacing={4} alignY="center">
                            <view>
                                <text>{e.branch}</text>
                            </view>
                            <view>
                                <text>{e.timestamp}</text>
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
        </>
    );
}


const InstalledVersion = () => {
    const { isLoading, isError, error, data } = useQuery(queryOptions({
        queryKey: ["builds::active"],
        queryFn: async () => {
            const data = await invoke<Version | null>("fetch::install_build", {});
            return data;
        }
    }),);

    if (isLoading) return (
        <col>
            <text>Loading</text>
        </col>
    );


    if (isError) return (
        <col>
            <text>{error.message}</text>
        </col>
    );


    if (!data) return (
        <row>
            <text>Installed:</text>
            <space width={6} />
            <text>No Installed Version</text>
        </row>
    );

    return (
        <row>
            <text>Installed:</text>

            <space width={6} />

            <row>
                <view width="shrink">
                    <svg src={Network} width={18} height={18} />
                </view>
                <view padding={[4, 8]} style="roundedBox">
                    <text>
                        {data.version}
                    </text>
                </view>
            </row>
        </row>
    );
}
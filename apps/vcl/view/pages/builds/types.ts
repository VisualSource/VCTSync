export type GithubVersion = {
    tag_name: string;
    name?: string;
    prerelease: boolean;
    draft: boolean;
    target_commitish: string;
    published_at: string;
    assets: { node_id: string; name: string; browser_download_url: string; size: number; digest: string, content_type: string; }[];
}

export type Version = {
    id: string;
    version: string;
    source_url: string;
    content_type: "local" | "remote",
    timestamp: string;
    branch: string,
    digest: string;
    /** Bytes, when the source reports it. */
    size?: number;
    prerelease?: boolean;
}
export type CatalogModVersion = {
    name: string;
    full_name: string;
    icon: string;
    version_number: string;
    description: string;
    dependencies: string[]
    download_url: string;
    downloads: number;
    date_created: string;
    website_url: string;
    is_active: boolean;
    uuid4: string;
    file_size: number;
}

export type CatalogMod = {
    name: string;
    full_name: string;
    owner: string;
    package_url: string;
    donation_link: string;
    date_created: string;
    date_updated: string;
    uuid4: string;
    rating_score: number;
    is_pinned: boolean;
    is_deprecated: boolean;
    has_nsfw_content: boolean;
    categories: string[];
    versions: CatalogModVersion[]
}

/** Thunderstore V1 manifest.json at the zip root. */
export type ZipManifest = {
    name: string;
    version_number: string;
    website_url: string;
    description: string;
    dependencies: string[];
    author?: string;
}

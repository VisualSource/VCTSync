import { QueryClient } from "@tanstack/react-query";
import { DbClient } from "@tanstack/react-db";


export const queryClient = new QueryClient();
export const dbClient = new DbClient({
    queryClient
});
import { createRoot } from "iced-dom";
import { useState } from "react";
import { QueryClientProvider, QueryClient, useQuery } from "@tanstack/react-query";

const client = new QueryClient();

const App = () => {
    return (
        <QueryClientProvider client={client}>
            <View />
        </QueryClientProvider>
    );
}
fetch("");

new Response()

const View = () => {
    const { data, isLoading } = useQuery({
        queryKey: ["hello"],
        queryFn: () => {
            return new Promise<string>((ok) => setTimeout(() => ok("Hello"), 1000));
        }
    })
    const [count, setCount] = useState(0);
    return (
        <view width="fill" height="fill" alignX="center" alignY="center">
            <col>
                <text>Hello, From JS</text>
                <row>
                    <button onPress={() => { setCount(prev => prev - 1) }}>Sub</button>
                    <text>{count.toString()}</text>
                    <button onPress={() => { setCount(prev => prev + 1) }}>Add</button>
                </row>
                {isLoading && data ? <text>Loading</text> : <text>{data ?? "missing"}</text>}
            </col>
        </view>
    );
}

createRoot("main").render(<App />);
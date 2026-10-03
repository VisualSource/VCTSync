import { createRoot } from "react-iced-native";
import { useState } from "react";
import { QueryClientProvider, QueryClient, useQuery } from "@tanstack/react-query";

import Boxes from "./boxes.svg" with { type: "svg" };


for (const item in globalThis) {
    console.log("global:", item);
}

const client = new QueryClient();

const App = () => {
    return (
        <QueryClientProvider client={client}>
            <View />
        </QueryClientProvider>
    );
}

const View = () => {
    const { data, isLoading } = useQuery({
        queryKey: ["hello"],
        queryFn: async () => {
            const data = await fetch("https://jsonplaceholder.typicode.com/todos/1");

            const content = await data.json() as { userId: number; id: number; title: string; completed: boolean };

            return content;
        }
    })
    const [count, setCount] = useState(0);
    const [text, setText] = useState("");
    return (
        <view width="fill" height="fill" alignX="center" alignY="center">
            <col>
                <svg src={Boxes} height={10} width={10} />

                <hr />

                <text>Hello, From JS</text>
                <row>
                    <button onPress={() => { setCount(prev => prev - 1) }}>Sub</button>
                    <text>{count.toString()}</text>
                    <button onPress={() => { setCount(prev => prev + 1) }}>Add</button>
                </row>

                <input type="text" value={text} onChange={(ev) => setText(ev.value)} placeholder="Enter text" />

                <tooltip>
                    <button>Hello</button>
                    <text>I'm a tool tip here</text>
                </tooltip>


                {isLoading ? <col /> : data ? (
                    <col>
                        <text>{data?.title}</text>
                        <text>{data?.completed ? "uncompleted" : "Completed"}</text>
                    </col>
                ) : <col />}
            </col>
        </view>
    );
}

createRoot("main").render(<App />);
import { createRoot } from "iced-dom";
import { useState } from "react";

const View = () => {
    const [count, setCount] = useState(0);
    return (
        <view height="full" width="full" alignX="center" alignY="center">
            <col>
                <text>Hello, From JS</text>
                <row>
                    <button onPress={() => { setCount(prev => prev - 1) }}>Sub</button>
                    <text>{count.toString()}</text>
                    <button onPress={() => { setCount(prev => prev + 1) }}>Add</button>
                </row>
            </col>
        </view>
    );
}

createRoot("main").render(<View />);
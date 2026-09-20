import { createRoot, createElement } from "iced-dom";

console.log(`
    Host env method and objects,
    window ${typeof window}
    self ${typeof self}    
    globalThis ${typeof globalThis}
    setTimeout ${typeof setTimeout}
    setInterval ${typeof setInterval}
    setImmediate ${typeof setImmediate}
    clearTimeout ${typeof clearTimeout}
    clearInterval ${typeof clearInterval}
    clearImmediate ${typeof clearImmediate}
    queueMicrotask ${typeof queueMicrotask}
    iced           ${typeof iced}
    iced.comment_tree ${typeof iced.comment_tree}
    createRoot  ${typeof createRoot}
    createElement ${typeof createElement}
`);

const View = () => {
    return createElement("view", {
        height: "full",
        width: "full",
        alignX: "center",
        alignY: "center",
        children: [
            createElement("text", {
                children: ["Hello, from JS"]
            })
        ]
    })
}

createRoot("main").render(View());
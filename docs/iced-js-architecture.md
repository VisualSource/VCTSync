# Rendering a React tree into iced widgets with rquickjs

**Design note — 2026-09-19**
**Subject:** `crates/iced-js` — running a user `.tsx` script in QuickJS via rquickjs 0.13, reconciling it with a custom React renderer, and converting the committed tree into `iced::Element`.

---

## Goal

An iced app points at a script path. The script renders through a custom React reconciler
(`js/iced-dom.ts`) whose host instances are plain data. Rust receives the committed tree and
turns it into iced widgets, which the app returns from `view()`.

A JS-rendered region is a *surface*, not the whole window: it composes inside an ordinary iced
tree, and an app may host several at once on one shared JS runtime.

As of this note the crate is a sketch: `IcedJsRuntime` is an empty struct, the `HostConfig`
methods are stubs, and the one test declares a module outside `async_with` (which cannot
compile — `Module::declare` takes `Ctx<'js>`, not `AsyncContext`) and calls `eval_file` with no
path argument.

## How React host renderers work

The reconciler is identical in react-dom, react-native and here. Only the `HostConfig` differs.
This section is the mental model the rest of the note assumes.

### The script runs once and exits

In a browser:

```js
import { createRoot } from 'react-dom/client';
createRoot(document.getElementById('app')).render(<App />);
```

The module body runs to completion and the call stack empties. The script is over; nothing is
looping. What keeps the page interactive is that the *host* — the browser — keeps the JS runtime
alive and re-enters it when something happens.

Here, Rust is the host. The module body finishes, the stack empties, and the runtime survives
because the subscription task owns the `AsyncRuntime` and `tokio::spawn(rt.drive())` keeps
polling it. Drop those and everything goes away; keep them and the context, module registry,
globals and fiber tree persist.

So the script exiting is normal and expected. What matters is what stays *reachable* and what
can *re-enter*.

### What keeps the tree alive

After the body returns, the fiber tree survives only if something still points at it. react-dom
gets this for free: DOM nodes carry `__reactFiber$…` back-pointers and the DOM is rooted in
`document`.

There is no DOM here. Host instances are plain objects that Rust converts to `tree::Node` and
discards. The **only** thing keeping the fiber tree alive is the root handle from
`createContainer`. So `iced-dom.ts` must retain it in module scope:

```ts
const roots = new Map<number, OpaqueRoot>();   // module scope = rooted by the module registry
```

Module-level bindings stay reachable from QuickJS's module registry for the life of the context.
The callback registry needs the same treatment. Its closures capture component scope and would
keep fibers alive incidentally — do not rely on that; root the container explicitly.

### Mutation vs persistence

**react-dom** sets `supportsMutation: true`. React walks the fiber diff and mutates the real DOM
in place (`appendChild`, `setAttribute`, `removeChild`). The DOM *is* the retained tree and the
browser paints from it. Nothing is handed back.

**react-native** (Fabric) uses immutable shadow trees. Each commit clones the changed spine and
produces a *new* tree, handed to the native side to lay out and render; the old tree is dropped.

**This crate is the second shape**, which is why `supportsPersistence: true` is correct. React
produces a fresh immutable tree per commit, `replaceContainerChildren` hands it over, and Rust
converts it to owned `Node`s.

This fits iced well. `view()` already rebuilds its entire `Element` tree every redraw and diffs
internally, so there are two independent rates — React commits (rare, on state change) and
`view()` calls (every frame) — with the `Arc<Node>` between them as the cache.

### Re-entry: the three doors

Only three things can re-enter JS after the script exits:

1. **A host callback** — `dispatch(id, payload)`, invoked from Rust when an iced widget fires.
   This initiates almost everything.
2. **A timer** — `setTimeout` firing, i.e. a `Ctx::spawn`ed future completing.
3. **A microtask** — a promise resolving.

Timers and microtasks are not what keep the runtime alive; Rust is. What they do is let React
**defer and batch** work after a re-entry. `setState` does not render — it marks a fiber dirty
and schedules. The scheduler then comes back on a later tick with an empty stack, so it can
time-slice and coalesce several `setState` calls from one event into a single commit.

Without them, `setState` marks work pending and nothing ever performs it: React silently stops.
That is the characteristic failure mode to watch for.

### Two schedulers, and the globals they need

They hit different host APIs, so both matter:

- **`HostConfig.scheduleMicrotask`** — the reconciler uses this to flush sync work at the end of
  the current microtask checkpoint.
- **The `scheduler` package** (`unstable_scheduleCallback`) — the concurrent work loop. It is a
  dependency of react-reconciler, so esbuild pulls it in automatically.

`scheduler@0.28.0` picks its work-loop pump from a three-step fallback
(`scheduler/cjs/scheduler.production.js:191-206`):

```js
if (typeof localSetImmediate === "function")      // 1. setImmediate
else if (typeof MessageChannel !== "undefined")   // 2. MessageChannel
else localSetTimeout(performWorkUntilDeadline, 0) // 3. setTimeout(fn, 0)
```

QuickJS has neither `setImmediate` nor `MessageChannel`, so it lands on `setTimeout(fn, 0)`.
That works, but **providing `setImmediate` is worthwhile** — it fires on every work slice, and a
`Ctx::spawn` of a ready future is cheaper than a `tokio::time::sleep(0)`.

For timing it uses `performance.now()` when present and falls back to `Date.now()`
(`scheduler.production.js:61-72`). QuickJS has `Date`, so this works by default, but the
scheduler compares against a 5 ms frame budget and `Date.now()` only has 1 ms resolution.
A real `performance.now()` backed by `Instant` makes time-slicing behave considerably better.

### One full cycle

```
user clicks
  → iced emits JsEvent::Callback(cb_id, payload)
  → update() sends JsCommand::Dispatch over the channel
  → subscription task: ctx.async_with(|ctx| dispatch.call((cb_id, payload)))
      → JS callback runs → setState → fiber marked dirty → scheduler.scheduleCallback
      → JS returns; stack empties
  → rt.drive() polls the spawned setImmediate/setTimeout future
      → performWorkUntilDeadline → reconcile → commit
      → replaceContainerChildren → container.commit(children)
      → __iced_commit(root_id, children) → owned tree::Node → sender.send()
  → iced delivers JsEvent::Committed { root_id, tree }
  → update() inserts into host.trees → iced redraws
  → view() → iced_js::view(&host, root_id) walks the Node tree → Element
```

`rt.drive()` sits in the middle of that chain. Without it everything stops after `setState`,
with work scheduled and nothing to run it. It is the most load-bearing line in the setup.

### Where the script is started

Not `view()` — it runs many times a second and must stay pure. Not `new()` — it cannot await,
and the runtime must outlive a single call. Not a `Task` — a `Task` runs once and yields one
message, whereas the runtime must live for the app's lifetime and emit a message per commit.

**A `Subscription`**, which is exactly a long-lived stream of messages kept alive as long as
`subscription()` returns the same identity. See implementation step 4.

## The three constraints

Everything below follows from these.

**1. `Element` cannot hold JS values.** `view(&self) -> Element<'_, Message>` is synchronous,
runs on the UI thread every redraw, and can neither await nor take the QuickJS lock. Every
`rquickjs` handle is `'js`-bound and valid only under that lock.

**2. `Persistent<T>` is not `Send`.** It holds a `*mut qjs::JSRuntime` plus a raw `JSValue` and
carries no `unsafe impl Send` (`rquickjs-core/src/persistent.rs:37`). Enabling the `parallel`
feature does not change this. So a `Persistent<Function>` cannot cross into an iced `Message`
or `Task`.

**3. Modules load once per context.** QuickJS caches modules by name, and React's reconciler
holds module-level state (fiber roots, the container). Re-evaluating per render would create a
second React instance and reset all component state.

Therefore: JS never enters `view()`, JS handles never leave the runtime task, and the JS
environment is built exactly once. The boundary carries owned Rust data and integer ids only.

## Architecture

```
Subscription task (owns ONE AsyncRuntime, drives it; N roots inside it)
  │  JS renders → container.commit(children)
  │  Rust host fn converts JS tree → owned tree::Node
  ├────── mpsc::Sender<JsEvent> ──────▶ update(): host.trees.insert(root_id, Arc<Node>)
  │         Committed { root_id, tree }  view(): walk Arc<Node> → Element
  ◀────── mpsc::Sender<JsCommand> ───── update(): Mount/Unmount/Dispatch(cb_id, payload)
     dispatch cb_id → JS-side callback registry
```

Callbacks stay **inside JS**. `iced-dom.ts` keeps a `Map<number, Function>`; props crossing to
Rust carry only a `u64` id. This sidesteps constraint 2 entirely and keeps `JsEvent` trivially
`Send + Clone`.

### Why `AsyncRuntime` and not `Runtime`

The difference is not really "one can be awaited." `AsyncRuntime::new()` builds its `Opaque`
with a spawner — a futures scheduler living inside the JS runtime — where `Runtime::new()`
leaves it `None`. `Ctx::spawn` pushes into that spawner and panics on a sync runtime
("tried to use async function in non async runtime", `runtime/opaque.rs:153`). Every Rust host
function that returns a JS promise, `setTimeout` included, goes through `Ctx::spawn`. React's
scheduler needs `setTimeout`, so the async runtime is mandatory.

`AsyncRuntime` also adds `idle()` and `drive()`, which have no sync counterpart. Spawned
futures make **no progress** without one of them — see the `no_drive` vs `idle` tests in
`rquickjs-core/src/runtime/async.rs`.

Threading is a separate axis: `Send`/`Sync` come from the `parallel` feature, not from
`AsyncRuntime`. `parallel` is enabled in `crates/iced-js/Cargo.toml`, which is what makes
`tokio::spawn(rt.drive())` legal alongside iced's multithreaded executor.

## Public API: multiple surfaces

A JS-rendered region should be composable inside an ordinary iced tree, not the whole window:

```rust
fn view(&self) -> Element<'_, Message> {
    column![
        row![text("some text")],
        iced_js::view(&self.js, "user_script1.js").map(Message::Js),
    ].into()
}
```

This mirrors React Native's **surfaces**: a brownfield app embeds several independent RN views
inside native screens, each with its own `createContainer` root, all sharing one JS runtime and
one React instance. The sharing is the point — see "One runtime, N roots" below.

### `iced_js::view` is a function, not a `Widget`

A custom `Widget` impl is only needed for custom layout or drawing. This produces a tree of
ordinary iced widgets, so it is just a converter:

```rust
pub fn view<'a>(host: &'a Host, id: &str) -> Element<'a, JsEvent>
```

`text()` takes `impl IntoFragment<'a>` and returns `Text<'a, …>`, so the `Element` borrows
directly out of the cached `Node` tree — no per-frame cloning. Returning `Element<'a, JsEvent>`
and letting the caller `.map(Message::Js)` keeps the crate independent of the app's message type.

Note that the host has to be passed in. `view(&self)` is the only thing holding app state, so a
bare `iced_js_view("user_script1.js")` would have nothing to read from.

`view` must render a placeholder when the id has no committed tree yet.

### Mounting is driven from `update()`, not from `view()`

`view()` runs every frame and must stay pure, so a surface cannot load itself by being rendered.
Mount explicitly, where side effects already live:

```
subscription boots runtime → JsEvent::Ready(Sender<JsCommand>)
  → update() sends JsCommand::Mount { root_id, path } per wanted script
  → worker loads module, createContainer, renders
  → commits return as JsEvent::Committed { root_id, tree }
```

For a static set, take it at construction (`Host::new(["a.js", "b.js"])`) and let `Ready` flush
the mounts. For conditional surfaces — a tab, a panel — expose `host.mount(path) -> Task` and
`host.unmount(root_id) -> Task`, called from `update()` when the tab changes. That is natural,
since the tab switch is already a message.

### One runtime, N roots

Do not key the subscription on the script, and do not spawn a runtime per surface.

N `AsyncRuntime`s means N QuickJS instances, N copies of React instantiated, N `drive()` tasks
and no shared module cache. One runtime with N roots gives one React, one scheduler, one module
registry.

Subscription identity is a hash, so if the hashed data were the script set, adding a second
surface would tear down and restart the whole runtime — losing the first surface's state. Keep
the identity stable (the generation counter below) and mount over the `JsCommand` channel.

A useful consequence: the callback registry in `iced-dom.ts` is module-global, so **callback ids
are unique across all roots**. `dispatch(cb_id, payload)` needs no root id, and
`JsEvent::Callback(u64, Payload)` stays flat.

### Two consequences of sharing a runtime

**Module state is shared.** If two surfaces both import `./store.js`, they get the *same
instance*, evaluated once. That is either a feature (a store shared across surfaces) or a
surprise (one surface mutating another's state). Decide deliberately — isolation would require
separate contexts, and therefore separate React instances.

**iced widget state survives commits that keep the tree shape.** `Widget` has
`tag()`/`state()`/`diff()` (`iced_core/src/widget.rs:78-95`), so iced diffs by position and type;
scroll positions and text cursors persist across a React commit with stable structure, and reset
when the shape changes — the same semantics as React against the DOM. Mapping React's `key` onto
iced's `Id` where widgets support it would later give explicit control.

## Hot reload and module caching

Per the model above, Rust evaluates each mounted script exactly once and re-evaluates nothing
thereafter. Hot-reloading an edited file is the single case that legitimately re-runs a module,
and QuickJS's caching works against you: re-declaring the same module name will not re-execute
the body — you get the cached module, or a duplicate `JSModuleDef` under an ambiguous name.

Keying module names by `root_id` (step 3) partly covers this, since remounting an edited file
under a fresh id produces a fresh name. But it does not cover the script's *imports*: a shared
`./store.js` stays cached at its own name, so the reloaded surface picks up stale module state.

The reliable fix is to rebuild the whole context, and the subscription gives that almost for
free. iced diffs subscriptions by identity every update cycle, so bumping the generation counter
makes iced drop the old stream — tearing down that runtime — and start a fresh one with a new
context. It also discards stale React state, which is what you want on reload.

## Loading: `embed!` + `Bundle` for the static half

`embed!` compiles to bytecode. The macro spins up a throwaway `Runtime` *at Rust compile time*,
runs `Module::declare(..).write(WriteOptions::default())` per file, and emits the bytecode as a
static `phf::Map` (`rquickjs-macro/src/embed.rs:93-108`). With the `phf` feature on, `Bundle`
resolves to `Bundle<PhfBundleData<&'static [u8]>>` and its `Loader` impl calls
`unsafe Module::load(ctx, bytecode)` — no parsing at startup.

This matters: parsing the full React + reconciler bundle on every launch is real work, and this
moves it to build time.

```rust
static BUNDLE: Bundle = embed! {
    "iced-dom": "js/dist/iced-dom.js",
};
rt.set_loader(BUNDLE, BUNDLE).await;   // before any module load
```

Caveats:

- `embed!` only compiles; it does not resolve imports. The esbuild step must still produce one
  self-contained ESM file per entry.
- Bytecode is locked to this exact QuickJS build. Consistent by construction (the macro uses the
  same `rquickjs-core` the crate links), but it cannot be cached across rquickjs upgrades.
- `WriteOptions::default()` keeps source and debug info, so stack traces survive. Leave it.

**The user script cannot use `embed!`** — its path is a runtime value, which is the point of the
crate. Read it with `tokio::fs` and `Module::declare` it directly; its
`import { createRoot } from "iced-dom"` still resolves through the runtime resolver, so `BUNDLE`
covers it.

Avoid `ScriptLoader`/`FileResolver` here. `Loader::load` is a sync trait method, so
`ScriptLoader` performs a blocking `std::fs::read` (`rquickjs-core/src/loader/script_loader.rs:52`)
on the runtime thread with the lock held. If relative imports *between* user scripts are wanted
later, `Resolver` and `Loader` are implemented for tuples with fallthrough on
`Error::Resolving`/`Error::Loading` (`rquickjs-core/src/loader.rs:246-300`), so
`set_loader((BUNDLE, FileResolver::…), (BUNDLE, ScriptLoader::…))` composes — at the cost of
that blocking read.

## Implementation

### 1. Owned tree type — `src/render.rs`

```rust
pub enum Tag { Row, Col, Text, Button, Svg, Scroll }

pub enum Node {
    Element { tag: Tag, props: Props, children: Vec<Node> },
    Text(Box<str>),
}

pub enum PropValue { Str(Box<str>), Num(f64), Bool(bool), Callback(u64) }
pub type Props = Vec<(Box<str>, PropValue)>;   // small; linear scan beats a HashMap here
```

No lifetimes, no `rquickjs` types. Conversion from the JS tree walks it once at commit time.

**Tags are an enum, not strings.** `view()` runs every frame, so a match on an enum is a jump
table where a string compare per node is not. Adding a tag becomes a compile error everywhere it
must be handled rather than a silent fallthrough. Most importantly, an unrecognised tag is
rejected **at commit time**, where the `root_id` is still in hand and the failure can surface as
`Event::Error`; by `view()` there is nowhere for an error to go. The same argument applies to
prop keys, though there are enough of them that it is more of a judgement call.

**Strings stay `Box<str>`.** Text content and string prop values are freshly allocated from JS on
every commit, so a `Cow<'static, str>` would always be `Owned` — no saving, and 24 bytes against
16. Note that iced's `Fragment<'a>` *is* `Cow<'a, str>` (`iced_core/src/text.rs:603`) and
`IntoFragment` is implemented for `&'a str`, `&'a String`, `String` and `Cow` — but **not for
`&Box<str>`**. So render through a deref:

```rust
Node::Text(txt) => iced::widget::text(&**txt).into(),
```

#### JS → Node conversion

`FromJs` is implemented for `StdString` (`rquickjs-core/src/value/convert/from.rs:39`) but not for
`Box<str>` — `from_js` returns `Self`, so the type must be `Sized`, and `str` is not. (There *is*
`impl FromJs for Box<T> where T: FromJs` at `from.rs:297`, but that needs a sized `T`.) So go
through `String` and convert:

```rust
let tag: Box<str> = obj.get::<_, String>("type")?.into_boxed_str();
```

This does not double-allocate. `String::to_string()` ends in `str::from_utf8(bytes).map(|s| s.into())`
(`rquickjs-core/src/value/string.rs:23`), and `&str → String` allocates exactly `len` bytes with
no spare capacity, so `into_boxed_str()` finds `capacity == len` and does not realloc.

**Write a plain recursive function, not a `FromJs` impl.** The `FromJs` derive only covers
plain-data structs, and while the trait can be hand-implemented for an enum, two things argue
against it here:

- `FromJs` must return `rquickjs::Error`, whose `FromJs` variant is
  `{ from: &'static str, to: &'static str, message: Option<String> }`. `Event::Error` wants the
  offending tag, the script, and the path within the tree; through `rquickjs::Error` that
  collapses into one string.
- The tree comes from user JS, so recursion needs a **depth guard** — an over-nested tree
  overflows the stack inside a QuickJS callback, which aborts rather than raising a catchable
  error. A plain function can thread a depth counter; `FromJs` has nowhere to put one.

Parse `Tag` in this same function, so an unrecognised tag is rejected at commit time while the
`root_id` is still in hand.

#### Known optimizations, deliberately deferred

Persistent mode reuses unchanged instances *by reference* across commits, but the conversion
above re-walks the whole JS tree every commit regardless. Two ways to exploit the sharing, both
worth doing only once the tree shape and update pattern are known:

- **Id + memo table.** Stamp a monotonic `__id` in `createInstance`/`cloneInstance`; Rust keeps
  `HashMap<u64, Arc<Node>>` and, on hitting a cached id, reuses the `Arc` without descending.
  Little machinery, gets the sharing.
- **Rust-backed instances.** `#[rquickjs::class] IcedNode { tag, props, children: Vec<Arc<Node>> }`,
  so an unchanged subtree *is* the same `Arc` and commit converts only the changed spine. It also
  moves tag/prop validation into `createInstance`, where it throws during React's render and so
  gets a component stack and error-boundary handling — neither of which exists at commit time.

  If this is taken up, the rule is **keep JS values out of the class**. `JsClass` requires
  `Trace + JsLifetime` (`rquickjs-core/src/class.rs:87`); both derive trivially for pure-Rust
  fields, whereas storing `Class<'js, IcedNode>` children means tracing each one by hand, and an
  incorrect `Trace` leaks silently — the cycle collector simply never breaks the cycle. Note also
  that `Class` is `Rc<RefCell<_>>`-like, so `try_borrow_mut()` rather than `borrow_mut()`, and
  that instances become opaque to `console.log`, which bites while the reconciler is still being
  brought up.

### 2. Host globals — `src/js_host/` (`console.rs`, `timers.rs`, `react_reconciler.rs`)

QuickJS ships none of these; React's scheduler will not run without them.

| Global | Implementation |
| --- | --- |
| `setTimeout` / `clearTimeout` | `Ctx::spawn` + `tokio::time::sleep`, keyed by id |
| `setImmediate` | `Ctx::spawn` of a ready future — optional, but the scheduler's hot path |
| `queueMicrotask` | resolve an already-settled promise, chain onto it |
| `performance.now()` | elapsed from a monotonic `Instant` — optional; `Date.now()` is the fallback |
| `__iced_commit(tree)` | convert to `tree::Node`, push into `mpsc::Sender<Message>` |
| `console.log` | do this first; debugging without it is miserable |

`setImmediate` and `performance.now` are optional only in the sense that the `scheduler` package
degrades to `setTimeout(fn, 0)` and `Date.now()` without them — see "Two schedulers" above for
why both are worth providing anyway.

With `parallel` on, `ParallelSend: Send` (`rquickjs-core/src/markers.rs`), so every closure
handed to `Func::from` must be `Send`. A captured `mpsc::Sender` is; a `Persistent` is not.

### 3. Runtime setup — `src/runtime.rs`

```rust
let rt = AsyncRuntime::new()?;
rt.set_loader(BUNDLE, BUNDLE).await;
tokio::spawn(rt.drive());                       // Send only because `parallel` is on
let ctx = AsyncContext::full(&rt).await?;

ctx.async_with(async |ctx| host::install(&ctx)).await?;   // once, before any module runs

// then per JsCommand::Mount { root_id, path }:
let src = tokio::fs::read_to_string(&path).await?;
ctx.async_with(async |ctx| {
    let name = format!("user:{root_id}");
    let user = Module::declare(ctx.clone(), name, src)?;     // compiles only
    let (user, p) = user.eval()?;                            // now it runs
    p.into_future::<()>().await?;                            // MUST settle
    let mount: Function = user.get("default")?;
    mount.call::<_, ()>((root_id,))?;                        // script calls createRoot(root_id)
}).await;
```

Load-bearing details:

- `Module::declare` compiles only; `.eval()` executes (`rquickjs-core/src/value/module.rs:378`).
- `.eval()` returns a `Promise` even with no top-level await. Skip settling it and the body may
  not have run, so `get()` finds nothing. This is the easiest mistake to make here.
- All module work must be inside `async_with` — `Module::declare` needs `Ctx<'js>`.
- `set_loader` before the first import; globals before the first module body executes.

`iced-dom` is never declared explicitly. The first mounted script's `import` pulls it from
`BUNDLE`, and QuickJS caches it for every later mount in the same context.

Module names are keyed by `root_id` so two surfaces can load the *same* file as independent
roots without colliding in the module registry.

### 4. Subscription — `src/runtime.rs` (`js_worker`)

```rust
Subscription::run_with(self.generation, |gen| {
    iced::stream::channel(100, async move |sender| { /* runtime + command loop */ })
})
```

`run_with` takes `D: Hash` and a bare `fn(&D) -> S`, so the runtime is constructed *inside* the
stream. The hashed data is the **generation counter, not the script set** — see "One runtime, N
roots" for why. Bumping it restarts the runtime with a fresh context, which is the hot-reload
path.

The first message out is `JsEvent::Ready(Sender<JsCommand>)` (the standard iced worker pattern),
after which the task loops on incoming `JsCommand`s — `Mount`, `Unmount`, `Dispatch` — while
`__iced_commit` pushes `JsEvent::Committed { root_id, tree }` back out.

### 5. Tree → `Element` — `src/render.rs` and `src/view.rs`

Pure, synchronous, no JS. Match on `tag`, build the widget, recurse. Start with `text`, `col`,
`row`, `button`. The public entry point is `iced_js::view(&host, id) -> Element<'_, JsEvent>`,
which looks the tree up by `root_id` and renders a placeholder when there is not one yet.

The recursive helper must tie its output lifetime to the borrowed tree —
`fn render_tree<'a>(tree: &'a Node) -> Element<'a, Event>`. An unconstrained `'a` with a
plain `&Node` argument cannot be satisfied by any borrowed widget, whatever the string types are.

Reuse the **vocabulary**, not the code, from `crates/iced-xml/src/parser.rs`. It already fixes
tag names (`row`, `col`, `text`, `button`, `svg`, `scroll`) and their attribute sets (`padding`,
`spacing`, `width`, `height`, `clip`, `style`). Keeping both front-ends on one vocabulary means
the XML macro and JSX agree. It is a proc-macro, so none of it runs at runtime.

### 6. Finish the reconciler — `js/iced-dom.ts`

`supportsPersistence: true` / `supportsMutation: false` is the right choice for an immutable
retained tree — keep it.

- Widen `IcedProps` from `Record<string, string>` to allow numbers, booleans and callback ids.
- `createInstance` must return `{ type, props, children: [] }`. `appendInitialChild` already
  pushes into `.children`, but nothing creates that array today.
- `createTextInstance` → `{ text }`. `cloneInstance` → shallow copy, reusing `children` when
  `keepChildren`.
- `createContainerChildSet` / `appendChildToContainer` / `finalizeContainerChildren` /
  `replaceContainerChildren` are the persistent-mode commit path. `replaceContainerChildren` is
  where `container.commit(children)` fires.
- Add the callback registry: on `createInstance`, swap function props for freshly-minted ids and
  export `dispatch(id, payload)` for Rust to call. One module-global id counter, so ids stay
  unique across every root.
- `createRoot(rootId)` takes the id assigned by Rust, so each container's `commit` can tag its
  output `__iced_commit(rootId, children)`. Add `destroyRoot(rootId)` for `JsCommand::Unmount`.
- Hold both the roots and the callback registry in **module-scope** `Map`s. Per "What keeps the
  tree alive", the root handle is the only thing preventing the fiber tree from being collected
  once the script's stack unwinds.

### 7. Build step — `package.json`

`react` and `react-reconciler` are CJS with node resolution; QuickJS has neither. Add `esbuild`
and a `build:js` script emitting one self-contained ESM file to `js/dist/`, which `embed!` then
compiles to bytecode. Both packages are already installed; `react` is currently only a
`peerDependency`, so promote it to `dependencies`. Add `js/dist/` to `.gitignore`, and note that
`cargo build` now depends on `pnpm build:js` having run — a `build.rs` that shells out is an
option if that becomes annoying.

Keep `"moduleDetection": "force"` in `tsconfig.json`. The `export {}` it emits on every file is
correct. The exception seen earlier came from `EvalOptions::default()` setting `global: true`
(→ `JS_EVAL_TYPE_GLOBAL`, `rquickjs-core/src/context/ctx.rs:45-49`), where `export` is a syntax
error. Loading as a module makes it a non-issue; `EvalOptions { global: false, .. }` would also
fix it for plain eval.

### 8. Wire up — `src/host.rs` and `src/lib.rs`

`IcedJsRuntime` becomes `Host`, app-side state keyed by root rather than holding a single tree:

```rust
type RootId = String;                       // caller-chosen name, e.g. "sidebar"

pub struct Host {
    generation: u64,                        // subscription identity; bump to restart
    tx: Option<Sender<JsCommand>>,          // arrives with JsEvent::Ready
    trees: HashMap<RootId, Arc<Node>>,      // what `view` reads
    pending: Vec<(RootId, PathBuf)>,        // mounts requested before Ready landed
}

pub enum JsEvent {
    Ready(Sender<JsCommand>),
    Committed { root_id: RootId, tree: Arc<Node> },
    Callback(u64, Payload),
    Error { root_id: Option<RootId>, message: String },
}

pub enum JsCommand {
    Mount { root_id: RootId, path: PathBuf },
    Unmount(RootId),
    Dispatch(u64, Payload),
}
```

Surface: `Host::new(impl IntoIterator<Item = …>)`, `mount(path) -> Task<JsEvent>`,
`unmount(id) -> Task<JsEvent>`, `update(&mut self, JsEvent) -> Task<JsEvent>`,
`subscription(&self) -> Subscription<JsEvent>`, and the free function
`iced_js::view(&host, id) -> Element<'_, JsEvent>`.

`pending` matters: mounts requested before `Ready` arrives have nowhere to go, so queue them and
flush on `Ready`. `Error` carries an optional `root_id` because module-load failures belong to a
surface, while runtime failures do not.

## Verification

1. `cargo test -p iced-js` — a `#[tokio::test(flavor = "multi_thread")]` that boots the runtime,
   evaluates a module exporting `default = () => root.render(<text>hello</text>)`, awaits
   `rt.idle()`, and asserts the committed `Node` is `Element { tag: "text", .. }` with a single
   `Text("hello")` child. Exercises every seam except `view()`.
2. A test asserting `render::to_element` on that `Node` produces a widget without panicking.
3. A callback round-trip: commit a `button` with `onPress`, send `JsCommand::Dispatch(id)`,
   assert a second commit arrives with updated state — and that `useState` survived, which is
   what proves the module was not re-evaluated. This is the piece most likely to be subtly wrong.
4. Two surfaces: mount the same script twice under different `root_id`s, assert two independent
   commits arrive and that dispatching a callback on one leaves the other's tree untouched. This
   is what proves root isolation on a shared runtime.
5. `pnpm build:js && cargo run -p app` for a real window; confirm it renders, that clicking
   re-renders, and that a JS surface composes inside an ordinary `column!`/`row!`.

Use `multi_thread` in tests. With `parallel` on, `tokio::spawn(rt.drive())` needs a real
executor, and single-threaded flavors will mask scheduling bugs.

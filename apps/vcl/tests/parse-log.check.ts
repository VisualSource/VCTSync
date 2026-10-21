// Run: node --experimental-strip-types tests/parse-log.check.ts
import { parseLog } from "../view/pages/logs/parse.ts";
import assert from "node:assert";

const log = [
  "[Message:   BepInEx] BepInEx 5.4.23.2 - Void Crew",
  "[Error  :   BepInEx] Error loading [BetterScoop 0.4.1]: TypeLoadException",
  "  at BepInEx.Bootstrap.Chainloader.Start ()",
  "[Photon] not a level, so a continuation",
  "",
  "[Warning: Unity Log] missing script",
  "[Debug  :VoidCrewTerminus] Handshake ok\r",
  "[Fatal  :VoidCrewTerminus] boom",
].join("\n");

const e = parseLog(log);
assert.equal(e.length, 5);
assert.deepEqual(e[0], { level: "info", source: "BepInEx", message: "BepInEx 5.4.23.2 - Void Crew", trace: [] });
assert.equal(e[1].level, "error");
assert.equal(e[1].message, "Error loading [BetterScoop 0.4.1]: TypeLoadException");
assert.deepEqual(e[1].trace, ["  at BepInEx.Bootstrap.Chainloader.Start ()", "[Photon] not a level, so a continuation"]);
assert.equal(e[2].source, "Unity Log");
assert.equal(e[3].source, "VoidCrewTerminus");
assert.equal(e[3].level, "debug");
assert.equal(e[4].level, "error");
assert.deepEqual(parseLog("stray first line"), [{ level: "info", source: "", message: "stray first line", trace: [] }]);
console.log("parse ok");

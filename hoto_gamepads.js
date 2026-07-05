// Web gamepad backend for Hero of the Overworld.
//
// macroquad/miniquad ships no gamepad support, so the native build reads pads
// with gilrs and the web build reads the browser Gamepad API here. This registers
// a miniquad plugin exposing one function the Rust input module imports:
//
//   hoto_gamepads_poll() -> Uint8Array of (count * 7) bytes. For each connected
//   gamepad, seven 0/1 flags in the order the Rust side expects:
//     Up, Down, Left, Right, Confirm (A/south), Cancel (B/east), Menu (Start/Select)
//   i.e. the standard-mapping buttons and left stick already folded into our
//   logical buttons, so Rust just reads flags and derives press edges by frame.
//
// The browser only reveals a gamepad after the user presses a button on it (a
// gesture requirement), so pads pop in on first input — nothing to do here.
//
// Depends on sapp_jsutils.js (for `js_object`) and mq_js_bundle.js (for
// `miniquad_add_plugin`); load both before this file.
"use strict";

(function () {
    const N = 7; // logical buttons per pad; must match input.rs `N`
    const DEADZONE = 0.5; // matches STICK_DEADZONE on the native side

    function poll() {
        const pads =
            (navigator.getGamepads ? navigator.getGamepads() : []) || [];
        // Standard mapping: 0=A, 1=B, 8=Select, 9=Start, 12..15=dpad U/D/L/R;
        // axes[0]=left stick X (right +), axes[1]=left stick Y (down +).
        const out = [];
        for (let g = 0; g < pads.length; g++) {
            const gp = pads[g];
            if (!gp) {
                continue; // empty slots (disconnected pads) are skipped
            }
            const b = gp.buttons || [];
            const ax = gp.axes || [];
            const pressed = function (i) {
                return b[i] && b[i].pressed ? 1 : 0;
            };
            const lx = ax.length > 0 ? ax[0] : 0;
            const ly = ax.length > 1 ? ax[1] : 0;
            out.push(pressed(12) || ly < -DEADZONE ? 1 : 0); // Up
            out.push(pressed(13) || ly > DEADZONE ? 1 : 0); // Down
            out.push(pressed(14) || lx < -DEADZONE ? 1 : 0); // Left
            out.push(pressed(15) || lx > DEADZONE ? 1 : 0); // Right
            out.push(pressed(0)); // Confirm (A / south)
            out.push(pressed(1)); // Cancel (B / east)
            out.push(pressed(9) || pressed(8) ? 1 : 0); // Menu (Start / Select)
        }
        return new Uint8Array(out);
    }

    function register_plugin(importObject) {
        importObject.env.hoto_gamepads_poll = function () {
            return js_object(poll());
        };
    }

    miniquad_add_plugin({
        register_plugin: register_plugin,
        version: 1,
        name: "hoto_gamepads",
    });
})();

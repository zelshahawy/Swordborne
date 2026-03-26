import './restart-audio-context.js';
import init from './bevy_game.js';

const fsBtn = document.getElementById('fullscreen-btn');
fsBtn.addEventListener('click', () => {
    if (!document.fullscreenElement) {
        document.documentElement.requestFullscreen().catch(() => {});
    } else {
        document.exitFullscreen().catch(() => {});
    }
});
document.addEventListener('fullscreenchange', () => {
    fsBtn.textContent = document.fullscreenElement ? '✕ Exit' : '⛶ Fullscreen';
});

try {
    await init();
} catch (e) {
    // Bevy throws a control-flow exception on exit — ignore it
    if (e.message !== "Using exceptions for control flow, don't mind me. This isn't actually an error!") {
        throw e;
    }
}

const left = document.getElementById('info-panel-left');
const right = document.getElementById('info-panel-right');

const gl = document.createElement('canvas').getContext('webgl2');
if (gl) {
    const ext = gl.getExtension('WEBGL_debug_renderer_info');
    if (ext) {
        const vendor = gl.getParameter(ext.UNMASKED_VENDOR_WEBGL).toLowerCase();
        right.textContent = gl.getParameter(ext.UNMASKED_RENDERER_WEBGL);
        if (vendor.includes('intel')) {
            left.textContent = "Likely using an integrated GPU. Running slow? Make sure hardware acceleration is on.";
        } else if (vendor.includes('nvidia') || vendor.includes('amd') || vendor.includes('ati')) {
            left.textContent = "Likely using a dedicated GPU.";
        } else {
            left.textContent = "Could not detect GPU. Make sure hardware acceleration is on.";
        }
    } else {
        left.textContent = "Could not get GPU info. Make sure hardware acceleration is on.";
    }
} else {
    left.textContent = "WebGL2 not supported. Try a different browser.";
}

// Browsers block audio until a user interaction. Resume on first click/key.
const AudioContext = window.AudioContext || window.webkitAudioContext;
if (AudioContext) {
    const ctx = new AudioContext();
    const resume = () => { if (ctx.state === 'suspended') ctx.resume(); };
    document.addEventListener('click', resume);
    document.addEventListener('keydown', resume);
}

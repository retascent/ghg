// noinspection JSUnusedGlobalSymbols
export function remove_overlay() {
    let overlay = document.getElementById('loading_overlay');
    let fadeOutMs = parseInt(getComputedStyle(overlay).getPropertyValue('--fade-out').slice(0, -2)); // `ms` suffix

    overlay.style.opacity = '0';
    setTimeout(() => overlay.remove(), fadeOutMs);
}

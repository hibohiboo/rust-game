import("../pkg/index.js").catch(console.error);
const button = document.querySelector('#ui button');
const canvas = document.querySelector('#canvas');
button.addEventListener('click', () => {
    canvas.focus();
    canvas.dispatchEvent(new KeyboardEvent('keydown', {code: 'ArrowRight'}));
    button.remove();
});
document.querySelector('#jump').addEventListener('pointerdown', () => {
    canvas.focus();
    canvas.dispatchEvent(new KeyboardEvent('keydown', {code: 'Space'}));

});
document.querySelector('#jump').addEventListener('pointerup', () => {
    canvas.focus();
    canvas.dispatchEvent(new KeyboardEvent('keyup', {code: 'Space'}));
});
document.querySelector('#sliding').addEventListener('pointerdown', () => {
    canvas.focus();
    canvas.dispatchEvent(new KeyboardEvent('keydown', {code: 'ArrowDown'}));
});
document.querySelector('#sliding').addEventListener('pointerup', () => {
    canvas.focus();
    canvas.dispatchEvent(new KeyboardEvent('keyup', {code: 'ArrowDown'}));
});
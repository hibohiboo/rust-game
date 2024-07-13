import("../pkg/index.js").catch(console.error);
const button = document.querySelector('#ui button');
button.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    const canvas = document.querySelector('#canvas');
    canvas.focus();
    canvas.dispatchEvent(new KeyboardEvent('keydown', {code: 'ArrowRight'}));
    button.remove();
});

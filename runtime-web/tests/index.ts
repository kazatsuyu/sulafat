import { init, render } from "../ts/index.js";

init();

document.getElementById("render")?.addEventListener("click", render);

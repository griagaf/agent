const { listen } = window.__TAURI__.event;

const spot = document.getElementById("spot");

listen("pointer:target", ({ payload }) => {
  spot.style.left = `${payload.left}px`;
  spot.style.top = `${payload.top}px`;
  spot.style.width = `${payload.width}px`;
  spot.style.height = `${payload.height}px`;
  spot.hidden = false;
});

const { listen } = window.__TAURI__.event;

const spot = document.getElementById("spot");
// Стрелка стоит выше и левее цели; у края экрана ей там места нет.
const ARROW_ROOM = 76;

listen("pointer:target", ({ payload }) => {
  spot.style.left = `${payload.left}px`;
  spot.style.top = `${payload.top}px`;
  spot.style.width = `${payload.width}px`;
  spot.style.height = `${payload.height}px`;

  spot.classList.toggle("flip-x", payload.left < ARROW_ROOM);
  spot.classList.toggle("flip-y", payload.top < ARROW_ROOM);
  spot.dataset.level = payload.level;
  spot.hidden = false;
});

listen("pointer:level", ({ payload }) => {
  spot.dataset.level = payload;
});

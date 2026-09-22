const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const form = document.getElementById("form");
const question = document.getElementById("question");
const send = document.getElementById("send");
const trace = document.getElementById("trace");
const answer = document.getElementById("answer");
const showPointer = document.getElementById("show-pointer");
const hidePointer = document.getElementById("hide-pointer");

/** Те же события, что CLI печатает в консоль. */
const line = {
  said: (event) => event.text,
  tool_started: (event) => `${event.name}: ${JSON.stringify(event.input)}`,
  tool_finished: (event) => `${event.name} — готово, ${event.chars} символов`,
  tool_failed: (event) => `${event.name} — ошибка: ${event.error}`,
};

listen("agent:event", ({ payload }) => {
  const text = line[payload.kind]?.(payload);
  if (text === undefined) {
    return;
  }

  const item = document.createElement("li");
  item.textContent = text;
  item.classList.toggle("failed", payload.kind === "tool_failed");
  trace.append(item);
});

form.addEventListener("submit", async (event) => {
  event.preventDefault();

  trace.replaceChildren();
  answer.textContent = "";
  answer.classList.remove("failed");
  send.disabled = true;

  try {
    answer.textContent = await invoke("ask", { question: question.value });
  } catch (error) {
    answer.textContent = error.message ?? "Что-то пошло не так.";
    answer.classList.add("failed");
  } finally {
    send.disabled = false;
    question.focus();
  }
});

question.focus();

// Прямоугольник задаётся в физических пикселях — тех же, что вернёт UI Automation.
function rect() {
  const value = (id) => Number(document.getElementById(id).value);
  return {
    left: value("left"),
    top: value("top"),
    right: value("right"),
    bottom: value("bottom"),
  };
}

showPointer.addEventListener("click", () => invoke("show_pointer", { target: rect() }));
hidePointer.addEventListener("click", () => invoke("hide_pointer"));

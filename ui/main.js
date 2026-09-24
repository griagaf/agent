const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const POLL_MS = 400;

const form = document.getElementById("form");
const question = document.getElementById("question");
const send = document.getElementById("send");
const trace = document.getElementById("trace");
const answer = document.getElementById("answer");

const panel = document.getElementById("plan");
const intro = document.getElementById("plan-intro");
const stepList = document.getElementById("plan-steps");
const trouble = document.getElementById("plan-trouble");
const perform = document.getElementById("perform");
const skip = document.getElementById("skip");
const stop = document.getElementById("stop");
const planJson = document.getElementById("plan-json");
const runPlan = document.getElementById("run-plan");

let plan = null;
let current = 0;
let running = false;
// Опрос живёт до следующего перехода: иначе старый тик засчитает уже другой шаг.
let generation = 0;
// Переход занимает около секунды, и второй клик за это время перескочил бы шаг.
let busy = false;

/** Те же события, что CLI печатает в консоль. */
const line = {
  said: (event) => event.text,
  tool_started: (event) => `${event.name}: ${JSON.stringify(event.input)}`,
  tool_finished: (event) => `${event.name} — готово, ${event.chars} символов`,
  tool_failed: (event) => `${event.name} — ошибка: ${event.error}`,
  planned: (event) => `план на ${event.plan.steps.length} шагов`,
};

listen("agent:event", ({ payload }) => {
  const text = line[payload.kind]?.(payload);
  if (text !== undefined) {
    const item = document.createElement("li");
    item.textContent = text;
    item.classList.toggle("failed", payload.kind === "tool_failed");
    trace.append(item);
  }

  if (payload.kind === "planned") {
    start(payload.plan);
  }
});

form.addEventListener("submit", async (event) => {
  event.preventDefault();

  trace.replaceChildren();
  answer.textContent = "";
  answer.classList.remove("failed");
  await halt();
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

async function start(next) {
  plan = next;
  current = 0;
  running = true;

  intro.textContent = plan.intro;
  trouble.textContent = "";
  panel.hidden = false;
  render();

  await enter();
}

async function enter() {
  if (!running) {
    return;
  }
  if (current >= plan.steps.length) {
    await finish();
    return;
  }

  generation += 1;
  const mine = generation;
  busy = true;

  render();
  try {
    await invoke("show_step", { step: plan.steps[current] });
    trouble.textContent = "";
  } catch (error) {
    trouble.textContent = error.message ?? "Не получилось показать шаг.";
  } finally {
    busy = false;
    render();
  }

  if (mine !== generation) {
    return;
  }

  // Проверка, истинная ещё до шага, засчитала бы его мгновенно и человек бы его не увидел.
  if (await passes()) {
    trouble.textContent =
      "Этот шаг не получится засчитать сам: сделайте его и нажмите «Дальше».";
    return;
  }

  tick(mine);
}

async function passes() {
  try {
    return await invoke("check_step", { expect: plan.steps[current].expect });
  } catch {
    return false;
  }
}

async function tick(mine) {
  if (!running || mine !== generation) {
    return;
  }

  const done = await passes();

  if (!running || mine !== generation) {
    return;
  }
  if (done) {
    current += 1;
    await enter();
    return;
  }

  setTimeout(() => tick(mine), POLL_MS);
}

async function finish() {
  running = false;
  generation += 1;
  render();
  trouble.textContent = "Готово — все шаги сделаны.";
  await hidePointer();
}

async function halt() {
  if (!running) {
    return;
  }

  running = false;
  generation += 1;
  trouble.textContent = "Остановлено.";
  await hidePointer();
}

async function hidePointer() {
  try {
    await invoke("hide_pointer");
  } catch {
    // Указатель мог и не показываться — молчим.
  }
}

function render() {
  stepList.replaceChildren();

  plan.steps.forEach((step, index) => {
    const item = document.createElement("li");
    item.textContent = step.hint;

    if (index < current) {
      item.classList.add("done");
    } else if (index === current && running) {
      item.classList.add("active");
    }

    stepList.append(item);
  });

  perform.disabled = !running || busy;
  skip.disabled = !running || busy;
  stop.disabled = !running;
}

perform.addEventListener("click", async () => {
  if (!running || busy) {
    return;
  }

  busy = true;
  render();
  try {
    await invoke("perform_step", { step: plan.steps[current] });
  } catch (error) {
    trouble.textContent = error.message ?? "Не получилось сделать шаг.";
  } finally {
    busy = false;
    render();
  }
});

skip.addEventListener("click", async () => {
  if (!running || busy) {
    return;
  }

  current += 1;
  await enter();
});

stop.addEventListener("click", halt);

runPlan.addEventListener("click", async () => {
  await halt();

  try {
    await start(JSON.parse(planJson.value));
  } catch (error) {
    trouble.textContent = `План не разобрался: ${error.message}`;
    panel.hidden = false;
  }
});

question.focus();

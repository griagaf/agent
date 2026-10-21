import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import App from "./App.vue";

describe("главное окно", () => {
  it("показывает вопрос стартового экрана", () => {
    const wrapper = mount(App);

    expect(wrapper.get("h1").text()).toBe("Что сделаем вместе?");
  });
});

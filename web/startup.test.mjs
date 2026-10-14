import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import vm from "node:vm";

const html = await readFile(new URL("./index.html", import.meta.url), "utf8");
const source = html.match(/<script type="module">([\s\S]*?)<\/script>/)?.[1];
assert.ok(source, "Web 入口必须包含实际启动模块");
const wasm = Uint8Array.of(0, 97, 115, 109, 1, 0, 0, 0);

function element(textContent = "") {
  const listeners = new Map();
  return {
    textContent, hidden: false, disabled: false,
    addEventListener(type, callback, options) {
      listeners.set(type, { callback, once: options?.once });
    },
    async emit(type) {
      const listener = listeners.get(type);
      if (!listener) return;
      if (listener.once) listeners.delete(type);
      await listener.callback({ currentTarget: this });
    },
  };
}

async function harness({ download, init = async () => {}, probe = false } = {}) {
  const status = element(html.match(/id="status"[^>]*>([^<]*)</)?.[1]);
  const start = element();
  start.disabled = true;
  const canvas = element();
  const elements = new Map([["status", status], ["start", start], ["pvz-canvas", canvas]]);
  const errors = [];
  const reached = Promise.withResolvers();
  const release = Promise.withResolvers();
  const directory = probe ? "../target/web-probe" : "../target/web";
  const context = vm.createContext({
    document: { getElementById: (id) => {
      assert.ok(elements.has(id), `实际入口读取未知元素：${id}`);
      return elements.get(id);
    } },
    location: { search: probe ? "?probe=1" : "" }, URLSearchParams, WebAssembly,
    console: { error: (...args) => errors.push(args) },
    fetch: async (url) => {
      assert.equal(url, `${directory}/pvz_app_bg.wasm`);
      reached.resolve();
      await release.promise;
      return download ? download() : { ok: true, arrayBuffer: async () => wasm.buffer };
    },
  });
  const module = new vm.SourceTextModule(source, {
    context,
    importModuleDynamically: async (specifier) => {
      assert.equal(specifier, `${directory}/pvz_app.js`);
      const app = new vm.SyntheticModule(["default"], function () {
        this.setExport("default", async (options) => {
          assert.ok(options.module_or_path instanceof WebAssembly.Module);
          await init(options);
        });
      }, { context });
      await app.link(() => assert.fail("初始化替身没有静态依赖"));
      await app.evaluate();
      return app;
    },
  });
  await module.link(() => assert.fail("实际启动脚本没有静态依赖"));
  const evaluated = module.evaluate();
  await reached.promise;
  return { status, start, canvas, errors, evaluated, release: () => release.resolve() };
}

test("准备期点击画布仍显示状态且不能提前启动", async () => {
  const app = await harness();
  await app.canvas.emit("pointerdown");
  assert.equal(app.status.hidden, false);
  assert.equal(app.start.disabled, true);
  app.release();
  await app.evaluated;
  assert.equal(app.start.disabled, false);
});

test("准备期点击后下载失败仍显示中文错误", async () => {
  const app = await harness({ download: () => ({ ok: false }) });
  await app.canvas.emit("pointerdown");
  app.release();
  await app.evaluated;
  assert.equal(app.status.hidden, false);
  assert.match(app.status.textContent, /应用启动失败.*WASM/);
  assert.equal(app.start.disabled, true);
  assert.equal(app.errors.length, 1);
});

test("初始化失败及后续画布点击仍显示中文错误", async () => {
  const app = await harness({ init: async () => { throw new Error("原创初始化失败夹具"); } });
  app.release();
  await app.evaluated;
  await app.start.emit("click");
  await app.canvas.emit("pointerdown");
  assert.equal(app.status.hidden, false);
  assert.match(app.status.textContent, /应用启动失败/);
  assert.equal(app.start.hidden, false);
  assert.equal(app.errors.length, 1);
});

test("默认与探针入口成功后才由画布点击隐藏状态", async () => {
  for (const probe of [false, true]) {
    let initialized = 0;
    const app = await harness({ probe, init: async () => { initialized += 1; } });
    app.release();
    await app.evaluated;
    await app.start.emit("click");
    assert.equal(initialized, 1);
    assert.equal(app.start.hidden, true);
    assert.equal(app.status.hidden, false);
    assert.match(app.status.textContent, /已启动/);
    await app.canvas.emit("pointerdown");
    assert.equal(app.status.hidden, true);
    await app.start.emit("click");
    assert.equal(initialized, 1, "启动处理只执行一次");
    assert.deepEqual(app.errors, []);
  }
});

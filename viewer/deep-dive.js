const examples = Array.from(document.querySelectorAll("[data-example]"));
const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

function decodeBase64(value) {
  const binary = atob(value);
  const out = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    out[i] = binary.charCodeAt(i);
  }
  return out;
}

function packPalette(colors) {
  const out = new Uint8ClampedArray(256 * 4);
  for (let i = 0; i < 256; i += 1) {
    out[i * 4] = colors[i][0];
    out[i * 4 + 1] = colors[i][1];
    out[i * 4 + 2] = colors[i][2];
    out[i * 4 + 3] = 255;
  }
  return out;
}

function writePixel(target, targetIndex, palette, value) {
  const p = value * 4;
  const t = targetIndex * 4;
  target[t] = palette[p];
  target[t + 1] = palette[p + 1];
  target[t + 2] = palette[p + 2];
  target[t + 3] = 255;
}

function formatNumber(value, digits = 2) {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    return "n/a";
  }
  return value.toFixed(digits);
}

async function fetchPayload(query) {
  const response = await fetch(`/api/generate?${query}`, { cache: "no-store" });
  if (!response.ok) {
    let message = `Request failed (${response.status})`;
    try {
      const errorPayload = await response.json();
      message = errorPayload.error || message;
    } catch {
      // Static hosts without the Rust API will return a plain 404 page.
    }
    throw new Error(message);
  }
  return response.json();
}

class ArticleAnimation {
  constructor(root) {
    this.root = root;
    this.canvas = root.querySelector("canvas");
    this.ctx = this.canvas.getContext("2d", { alpha: false });
    this.meta = root.querySelector(".example-meta");
    this.title = root.dataset.title || "Generated example";
    this.query = root.dataset.query || "";
    this.view = root.dataset.view || "cortical";
    this.labUrl = root.dataset.lab || "index.html";
    this.fps = Number(root.dataset.fps || 18);
    this.loaded = false;
    this.loading = false;
    this.active = false;
    this.frame = 0;
    this.frameSize = 0;
    this.lastFrameAt = 0;
    this.payload = null;
    this.bytes = null;
    this.palette = null;
    this.sourceImage = null;
    this.sourceCanvas = document.createElement("canvas");
    this.retinalImage = null;
    this.retinalMap = null;
    this.retinalMapWidth = 0;
    this.retinalMapHeight = 0;
  }

  async load() {
    if (this.loaded || this.loading) {
      return;
    }
    this.loading = true;
    this.meta.textContent = "Generating frames...";
    try {
      const payload = await fetchPayload(this.query);
      this.install(payload);
      this.meta.innerHTML = `${this.summary(payload)} <a href="${this.labUrl}">Open in lab</a>`;
      this.meta.classList.remove("is-error");
      this.meta.classList.add("is-ready");
      this.loaded = true;
      this.resize();
      this.paint();
    } catch (error) {
      this.meta.classList.add("is-error");
      this.meta.textContent = `${this.title}: ${error.message}. The deep dive needs the Rust viewer server for live generated animations.`;
    } finally {
      this.loading = false;
    }
  }

  install(payload) {
    this.payload = payload;
    this.bytes = decodeBase64(payload.data_base64);
    this.palette = packPalette(payload.palette);
    this.frameSize = payload.width * payload.height;
    this.sourceCanvas.width = payload.width;
    this.sourceCanvas.height = payload.height;
    this.sourceImage = this.ctx.createImageData(payload.width, payload.height);
  }

  summary(payload) {
    const bits = [`${payload.width}x${payload.height}`, `${payload.frame_count} frames`];
    const params = payload.params || {};
    const generator = params.generator || "generated";
    bits.push(generator.replaceAll("_", " "));

    if (payload.rule) {
      bits.push(`${payload.rule.spatial_family}, ${payload.rule.response_mode}`);
      bits.push(`rT ${formatNumber(payload.rule.temporal_corr_t, 2)}`);
      bits.push(`r2T ${formatNumber(payload.rule.temporal_corr_2t, 2)}`);
    } else if (payload.calibration) {
      bits.push(`${payload.calibration.status}`);
      bits.push(`rendered ${payload.calibration.rendered_pattern}`);
      bits.push(`branch ${payload.calibration.selected_family}`);
    } else if (payload.planform) {
      bits.push(`rendered ${payload.planform.rendered_pattern || payload.planform.pattern}`);
    }

    return `${this.title}: ${bits.join(", ")}.`;
  }

  resize() {
    const rect = this.canvas.getBoundingClientRect();
    const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
    const size = Math.max(220, Math.round(rect.width * dpr));
    if (this.canvas.width === size && this.canvas.height === size) {
      return;
    }
    this.canvas.width = size;
    this.canvas.height = size;
    this.retinalImage = this.ctx.createImageData(size, size);
    this.retinalMap = null;
  }

  paint() {
    if (!this.loaded) {
      return;
    }
    this.resize();
    if (this.view === "retinal") {
      this.paintRetinal();
    } else {
      this.paintCortical();
    }
  }

  paintCortical() {
    const payload = this.payload;
    const imageBytes = this.sourceImage.data;
    const offset = this.frame * this.frameSize;
    for (let i = 0; i < this.frameSize; i += 1) {
      writePixel(imageBytes, i, this.palette, this.bytes[offset + i]);
    }

    const sourceCtx = this.sourceCanvas.getContext("2d", { alpha: false });
    sourceCtx.putImageData(this.sourceImage, 0, 0);

    this.ctx.imageSmoothingEnabled = false;
    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    this.ctx.drawImage(
      this.sourceCanvas,
      0,
      0,
      payload.width,
      payload.height,
      0,
      0,
      this.canvas.width,
      this.canvas.height,
    );
  }

  buildRetinalMap() {
    const payload = this.payload;
    const width = this.canvas.width;
    const height = this.canvas.height;
    const map = new Int32Array(width * height);
    const bounds = payload.retino_bounds;
    const params = payload.retino_params;
    const xRange = bounds.max_x - bounds.min_x;
    const yRange = bounds.max_y - bounds.min_y;
    const half = (payload.width * payload.cell_mm) / 2;

    for (let y = 0; y < height; y += 1) {
      const retinalY = bounds.max_y - ((y + 0.5) / height) * yRange;
      for (let x = 0; x < width; x += 1) {
        const retinalX = bounds.min_x + ((x + 0.5) / width) * xRange;
        const r = Math.hypot(retinalX, retinalY);
        if (r <= 0) {
          map[y * width + x] = -1;
          continue;
        }

        const theta = Math.atan2(retinalY, retinalX);
        const corticalX = (params.alpha / params.eps) * Math.log((params.eps / params.w0) * r);
        const corticalY = (params.beta * theta) / params.eps;
        const col = Math.floor((corticalX + half) / payload.cell_mm);
        const row = Math.floor((corticalY + half) / payload.cell_mm);

        if (row >= 0 && row < payload.height && col >= 0 && col < payload.width) {
          map[y * width + x] = row * payload.width + col;
        } else {
          map[y * width + x] = -1;
        }
      }
    }

    this.retinalMap = map;
    this.retinalMapWidth = width;
    this.retinalMapHeight = height;
  }

  paintRetinal() {
    if (
      !this.retinalMap ||
      this.retinalMapWidth !== this.canvas.width ||
      this.retinalMapHeight !== this.canvas.height
    ) {
      this.buildRetinalMap();
    }

    const target = this.retinalImage.data;
    const sourceOffset = this.frame * this.frameSize;
    const total = this.canvas.width * this.canvas.height;
    for (let i = 0; i < total; i += 1) {
      const sourceIndex = this.retinalMap[i];
      if (sourceIndex < 0) {
        const t = i * 4;
        target[t] = 7;
        target[t + 1] = 7;
        target[t + 2] = 7;
        target[t + 3] = 255;
      } else {
        writePixel(target, i, this.palette, this.bytes[sourceOffset + sourceIndex]);
      }
    }

    this.ctx.imageSmoothingEnabled = false;
    this.ctx.putImageData(this.retinalImage, 0, 0);
  }

  tick(now) {
    if (!this.loaded || !this.active || reduceMotion) {
      return;
    }
    if (now - this.lastFrameAt < 1000 / this.fps) {
      return;
    }
    this.lastFrameAt = now;
    this.frame = (this.frame + 1) % this.payload.frame_count;
    this.paint();
  }
}

const animations = examples.map((root) => new ArticleAnimation(root));

const observer = new IntersectionObserver(
  (entries) => {
    entries.forEach((entry) => {
      const animation = animations.find((item) => item.root === entry.target);
      if (!animation) {
        return;
      }
      animation.active = entry.isIntersecting;
      if (entry.isIntersecting) {
        animation.load();
      }
    });
  },
  { rootMargin: "900px 0px" },
);

animations.forEach((animation) => observer.observe(animation.root));

window.addEventListener("resize", () => {
  animations.forEach((animation) => animation.paint());
});

function animate(now) {
  animations.forEach((animation) => animation.tick(now));
  requestAnimationFrame(animate);
}

requestAnimationFrame(animate);

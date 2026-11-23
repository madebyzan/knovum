{
  const body = document.body;
  const header = document.querySelector(".site-header");
  const navigation = document.querySelector(".primary-nav");
  const menuButton = document.querySelector(".menu-toggle");
  const languageButton = document.querySelector(".language-toggle");
  const progressBar = document.querySelector(".reading-progress span");

  const CLOUDFLARE_WEB_ANALYTICS_TOKEN = "";

  const language = body.dataset.locale === "zh" ? "zh" : "en";
  const text = (en, zh) => (language === "zh" ? zh : en);
  const reducedMotion = globalThis.matchMedia("(prefers-reduced-motion: reduce)");

  body.classList.add("js-enabled");

  function installAnalytics() {
    if (!CLOUDFLARE_WEB_ANALYTICS_TOKEN || globalThis.location.protocol !== "https:") return;
    const beacon = document.createElement("script");
    beacon.type = "module";
    beacon.src = "https://static.cloudflareinsights.com/beacon.min.js";
    beacon.dataset.cfBeacon = JSON.stringify({ token: CLOUDFLARE_WEB_ANALYTICS_TOKEN });
    document.head.append(beacon);
  }

  function applyLanguage() {
    document.documentElement.lang = language === "zh" ? "zh-CN" : "en";

    for (const element of document.querySelectorAll("[data-en][data-zh]")) {
      element.textContent = element.dataset[language];
    }
    for (const element of document.querySelectorAll("[data-en-label][data-zh-label]")) {
      if (element.hasAttribute("data-carousel-toggle")) continue;
      element.setAttribute("aria-label", text(element.dataset.enLabel, element.dataset.zhLabel));
    }
    if (languageButton) {
      languageButton.textContent = language === "en" ? "中文" : "EN";
      languageButton.setAttribute("aria-label", text("Switch to Chinese", "切换到英文"));
    }
    syncMenuLabel();
  }

  function switchLanguage(next) {
    const brand = document.querySelector(".brand");
    if (!brand) return;
    const localizedRoot = new URL(brand.getAttribute("href"), globalThis.location.href).pathname.replace(
      /index\.html$/,
      "",
    );
    const siteRoot = localizedRoot.replace(/zh\/$/, "");
    const route = globalThis.location.pathname.slice(localizedRoot.length);
    globalThis.location.assign(siteRoot + (next === "zh" ? "zh/" : "") + route);
  }

  /* ------------------------------ navigation ----------------------------- */

  function syncMenuLabel() {
    if (!menuButton) return;
    const open = menuButton.getAttribute("aria-expanded") === "true";
    menuButton.setAttribute(
      "aria-label",
      open ? text("Close navigation", "关闭导航") : text("Open navigation", "打开导航"),
    );
  }

  function setMenu(open) {
    if (!navigation || !menuButton) return;
    navigation.classList.toggle("is-open", open);
    menuButton.setAttribute("aria-expanded", String(open));
    body.style.overflow = open ? "hidden" : "";
    if (!open) navigation.querySelectorAll("details[open]").forEach((item) => item.removeAttribute("open"));
    syncMenuLabel();
  }

  function initNavigation() {
    if (!menuButton || !navigation) return;
    menuButton.addEventListener("click", () => {
      setMenu(menuButton.getAttribute("aria-expanded") !== "true");
    });
    navigation.querySelectorAll("a").forEach((link) => {
      link.addEventListener("click", () => setMenu(false));
    });
    document.addEventListener("pointerdown", (event) => {
      if (menuButton.getAttribute("aria-expanded") === "true" && !header.contains(event.target)) {
        setMenu(false);
      }
      const openMenu = navigation.querySelector(".nav-menu[open]");
      if (openMenu && !openMenu.contains(event.target)) openMenu.removeAttribute("open");
    });
    document.addEventListener("keydown", (event) => {
      if (event.key !== "Escape") return;
      const wasOpen = menuButton.getAttribute("aria-expanded") === "true";
      setMenu(false);
      if (wasOpen) menuButton.focus();
    });
    globalThis.matchMedia("(min-width: 901px)").addEventListener("change", (event) => {
      if (event.matches) setMenu(false);
    });
  }

  function markCurrentPage() {
    const normalize = (pathname) => pathname.replace(/index\.html$/, "").replace(/\/$/, "");
    const currentPath = normalize(globalThis.location.pathname);
    for (const link of document.querySelectorAll(".primary-nav a[href]")) {
      const href = link.getAttribute("href");
      if (!href || href.startsWith("#")) continue;
      const target = new URL(href, globalThis.location.href);
      if (target.origin !== globalThis.location.origin) continue;
      const targetPath = normalize(target.pathname);
      if (targetPath === currentPath) link.setAttribute("aria-current", "page");
      else if (targetPath.endsWith("/guides") && currentPath.startsWith(targetPath + "/"))
        link.setAttribute("aria-current", "location");
      else link.removeAttribute("aria-current");
    }
  }

  /* ---------------------------- scroll effects ---------------------------- */

  function initScrollEffects() {
    let frameRequested = false;
    const update = () => {
      const scrollable = document.documentElement.scrollHeight - globalThis.innerHeight;
      const amount = scrollable > 0 ? Math.min(1, Math.max(0, globalThis.scrollY / scrollable)) : 0;
      if (progressBar) progressBar.style.transform = `scaleX(${amount})`;
      if (header) header.classList.toggle("is-scrolled", globalThis.scrollY > 8);
      frameRequested = false;
    };
    const request = () => {
      if (frameRequested) return;
      frameRequested = true;
      globalThis.requestAnimationFrame(update);
    };
    globalThis.addEventListener("scroll", request, { passive: true });
    globalThis.addEventListener("resize", request, { passive: true });
    update();
  }

  function initReveal() {
    const items = [...document.querySelectorAll("[data-reveal]")];
    const showAll = () => items.forEach((item) => item.classList.add("is-visible"));
    if (!("IntersectionObserver" in globalThis) || reducedMotion.matches) {
      showAll();
      return;
    }
    // Clip-path masked elements report an empty intersection rect while hidden,
    // so observe an unclipped anchor (the parent) instead of the element itself.
    const anchors = new Map();
    for (const item of items) {
      const anchor = item.dataset.reveal === "mask" && item.parentElement ? item.parentElement : item;
      const group = anchors.get(anchor);
      if (group) group.push(item);
      else anchors.set(anchor, [item]);
    }
    const observer = new IntersectionObserver(
      (entries, current) => {
        for (const entry of entries) {
          if (!entry.isIntersecting) continue;
          for (const item of anchors.get(entry.target) || []) item.classList.add("is-visible");
          current.unobserve(entry.target);
        }
      },
      { threshold: 0.1, rootMargin: "0px 0px -8% 0px" },
    );
    for (const anchor of anchors.keys()) observer.observe(anchor);
  }

  /* ------------------------------- carousel ------------------------------- */

  function initCarousels() {
    document.querySelectorAll("[data-image-carousel]").forEach((carousel) => {
      const slides = [...carousel.querySelectorAll(".image-carousel__slide, .carousel__slide")];
      const prevButton = carousel.querySelector("[data-carousel-prev]");
      const nextButton = carousel.querySelector("[data-carousel-next]");
      const toggleButton = carousel.querySelector("[data-carousel-toggle]");
      const counter = carousel.querySelector("[data-carousel-counter]");
      let active = null;
      let timer = null;
      let pausedByUser = reducedMotion.matches;
      let hovered = false;
      let focused = false;
      let inViewport = true;

      const visibleSlides = () => slides.filter((slide) => !slide.hidden && slide.isConnected);

      const syncToggle = () => {
        if (!toggleButton) return;
        toggleButton.dataset.paused = String(pausedByUser);
        toggleButton.setAttribute("aria-pressed", String(!pausedByUser));
        toggleButton.setAttribute(
          "aria-label",
          pausedByUser
            ? text("Resume automatic playback", "恢复自动播放")
            : text("Pause automatic playback", "暂停自动播放"),
        );
        const label = toggleButton.querySelector("span");
        if (label) label.textContent = pausedByUser ? text("Play", "播放") : text("Pause", "暂停");
      };

      const render = () => {
        const list = visibleSlides();
        if (!list.length) {
          carousel.classList.remove("is-ready");
          return;
        }
        if (!list.includes(active)) active = list[0];
        for (const slide of list) {
          const isActive = slide === active;
          slide.classList.toggle("is-active", isActive);
          slide.setAttribute("aria-hidden", String(!isActive));
        }
        carousel.classList.add("is-ready");
        if (counter) {
          const pad = (value) => String(value).padStart(2, "0");
          counter.textContent = `${pad(list.indexOf(active) + 1)} / ${pad(list.length)}`;
        }
        const multiple = list.length > 1;
        if (prevButton) prevButton.hidden = !multiple;
        if (nextButton) nextButton.hidden = !multiple;
        if (toggleButton) toggleButton.hidden = !multiple || reducedMotion.matches;
      };

      const show = (index) => {
        const list = visibleSlides();
        if (!list.length) return;
        active = list[((index % list.length) + list.length) % list.length];
        render();
      };

      const syncAutoplay = () => {
        if (timer !== null) {
          globalThis.clearInterval(timer);
          timer = null;
        }
        const idle =
          pausedByUser ||
          hovered ||
          focused ||
          !inViewport ||
          document.visibilityState === "hidden" ||
          reducedMotion.matches ||
          visibleSlides().length < 2;
        if (idle) return;
        timer = globalThis.setInterval(() => {
          const list = visibleSlides();
          show(list.indexOf(active) + 1);
        }, 6000);
      };

      prevButton?.addEventListener("click", () => {
        show(visibleSlides().indexOf(active) - 1);
        syncAutoplay();
      });
      nextButton?.addEventListener("click", () => {
        show(visibleSlides().indexOf(active) + 1);
        syncAutoplay();
      });
      toggleButton?.addEventListener("click", () => {
        pausedByUser = !pausedByUser;
        syncToggle();
        syncAutoplay();
      });
      carousel.addEventListener("mouseenter", () => {
        hovered = true;
        syncAutoplay();
      });
      carousel.addEventListener("mouseleave", () => {
        hovered = false;
        syncAutoplay();
      });
      carousel.addEventListener("focusin", () => {
        focused = true;
        syncAutoplay();
      });
      carousel.addEventListener("focusout", (event) => {
        focused = Boolean(event.relatedTarget && carousel.contains(event.relatedTarget));
        syncAutoplay();
      });
      document.addEventListener("visibilitychange", syncAutoplay);
      reducedMotion.addEventListener("change", () => {
        if (reducedMotion.matches) pausedByUser = true;
        syncToggle();
        render();
        syncAutoplay();
      });
      if ("IntersectionObserver" in globalThis) {
        new IntersectionObserver(([entry]) => {
          inViewport = entry.isIntersecting;
          syncAutoplay();
        }).observe(carousel);
      }
      carousel.refreshCarousel = () => {
        render();
        syncAutoplay();
      };

      syncToggle();
      render();
      syncAutoplay();
    });
  }

  /* --------------------------- deferred imagery ---------------------------- */

  function initOptionalImages() {
    for (const figure of document.querySelectorAll("[data-optional-image]")) {
      const image = figure.querySelector("img");
      const source = figure.dataset.optionalImage;
      if (!image || !source) continue;
      const carousel = figure.closest("[data-image-carousel]");
      image.addEventListener(
        "load",
        () => {
          figure.hidden = false;
          carousel?.refreshCarousel?.();
        },
        { once: true },
      );
      image.addEventListener(
        "error",
        () => {
          figure.remove();
          carousel?.refreshCarousel?.();
        },
        { once: true },
      );
      image.loading = "eager";
      image.src = source;
    }

    if (language !== "zh") return;
    for (const image of document.querySelectorAll("img[data-localized-src]")) {
      const localized = new Image();
      localized.decoding = "async";
      localized.addEventListener(
        "load",
        () => {
          image.src = localized.src;
        },
        { once: true },
      );
      localized.src = image.dataset.localizedSrc;
    }
  }

  /* --------------------------- release downloads --------------------------- */

  const RELEASE_LATEST_URL = "https://github.com/madebyzan/knovum/releases/latest";
  const RELEASE_ASSET_PATTERNS = {
    "macos-arm64": /_aarch64\.dmg$/i,
    "macos-x64": /_x64\.dmg$/i,
    "windows-x64": /_x64-setup\.exe$/i,
  };

  async function fetchLatestRelease() {
    const controller = new AbortController();
    const timeout = globalThis.setTimeout(() => controller.abort(), 5000);
    try {
      const response = await fetch("https://api.github.com/repos/madebyzan/knovum/releases/latest", {
        headers: { Accept: "application/vnd.github+json" },
        signal: controller.signal,
      });
      if (!response.ok) return null;
      return await response.json();
    } catch {
      return null;
    } finally {
      globalThis.clearTimeout(timeout);
    }
  }

  function applyReleaseLink(link, release, assets) {
    const asset = assets.find((candidate) =>
      RELEASE_ASSET_PATTERNS[link.dataset.releaseAsset]?.test(candidate.name || ""),
    );
    link.href = asset?.browser_download_url || release.html_url || RELEASE_LATEST_URL;
    link.dataset.releaseReady = String(Boolean(asset?.browser_download_url));
  }

  function applyFallbackLink(link) {
    link.href = RELEASE_LATEST_URL;
    link.dataset.releaseReady = "false";
  }

  async function initReleaseDownloads() {
    const links = [...document.querySelectorAll("[data-release-asset]")];
    if (!links.length || typeof globalThis.fetch !== "function") return;
    const release = await fetchLatestRelease();
    if (!release) {
      links.forEach(applyFallbackLink);
      return;
    }
    const assets = Array.isArray(release.assets) ? release.assets : [];
    for (const link of links) applyReleaseLink(link, release, assets);
    document.querySelectorAll("[data-release-version]").forEach((element) => {
      element.textContent = release.tag_name || "—";
    });
  }

  /* ------------------------------ small parts ------------------------------ */

  function initCopyButtons() {
    document.querySelectorAll("[data-copy]").forEach((button) => {
      button.addEventListener("click", async () => {
        const label = button.querySelector("[data-en]");
        const original = label?.dataset[language] || label?.textContent || "";
        try {
          await globalThis.navigator.clipboard.writeText(button.dataset.copy.replace(/&#10;/g, "\n"));
          if (label) label.textContent = text("Copied", "已复制");
        } catch {
          if (label) label.textContent = text("Select to copy", "请手动复制");
        }
        globalThis.setTimeout(() => {
          if (label) label.textContent = original;
        }, 1800);
      });
    });
  }

  function initFaq() {
    const items = [...document.querySelectorAll(".faq__item")];
    items.forEach((item) => {
      item.addEventListener("toggle", () => {
        if (!item.open) return;
        items.forEach((other) => {
          if (other !== item) other.open = false;
        });
      });
    });
  }

  installAnalytics();
  initNavigation();
  markCurrentPage();
  initScrollEffects();
  initReveal();
  initCarousels();
  initOptionalImages();
  initCopyButtons();
  initFaq();
  void initReleaseDownloads();
  document.querySelectorAll("[data-year]").forEach((element) => {
    element.textContent = String(new Date().getFullYear());
  });
  if (languageButton) {
    languageButton.addEventListener("click", () => switchLanguage(language === "en" ? "zh" : "en"));
  }
  applyLanguage();
  globalThis.requestAnimationFrame(() => body.classList.add("is-ready"));
}

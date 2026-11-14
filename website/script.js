{
  const root = document.documentElement;
  const body = document.body;
  const languageButton = document.querySelector(".language-toggle");
  const menuButton = document.querySelector(".menu-toggle");
  const navigation = document.querySelector(".primary-nav");
  const header = document.querySelector(".site-header");
  const progress = document.querySelector(".reading-progress span");
  body.classList.add("js-enabled");

  const language = body.dataset.locale === "zh" ? "zh" : "en";

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

  function updateMenuLabel() {
    if (!menuButton) return;
    const isOpen = menuButton.getAttribute("aria-expanded") === "true";
    const label =
      language === "en"
        ? isOpen
          ? "Close navigation"
          : "Open navigation"
        : isOpen
          ? "关闭导航"
          : "打开导航";
    menuButton.setAttribute("aria-label", label);
  }

  function applyPageLanguage() {
    root.lang = language === "zh" ? "zh-CN" : "en";

    document.querySelectorAll("[data-en][data-zh]").forEach((element) => {
      element.textContent = element.dataset[language];
    });

    if (languageButton) {
      languageButton.textContent = language === "en" ? "中文" : "EN";
      languageButton.setAttribute("aria-label", language === "en" ? "Switch to Chinese" : "切换到英文");
    }
    updateMenuLabel();
  }

  if (languageButton) {
    languageButton.addEventListener("click", () => switchLanguage(language === "en" ? "zh" : "en"));
  }

  function closeMenu() {
    if (!navigation || !menuButton) return;
    navigation.classList.remove("is-open");
    menuButton.setAttribute("aria-expanded", "false");
    navigation.querySelectorAll("details[open]").forEach((item) => item.removeAttribute("open"));
    updateMenuLabel();
  }

  if (menuButton && navigation) {
    menuButton.addEventListener("click", () => {
      const isOpen = menuButton.getAttribute("aria-expanded") === "true";
      menuButton.setAttribute("aria-expanded", String(!isOpen));
      navigation.classList.toggle("is-open", !isOpen);
      updateMenuLabel();
    });
    navigation.querySelectorAll("a").forEach((link) => link.addEventListener("click", closeMenu));
    document.addEventListener("keydown", (event) => {
      if (event.key === "Escape") closeMenu();
    });
  }

  const currentFile = globalThis.location.pathname.split("/").pop() || "index.html";
  document.querySelectorAll(".primary-nav a[href]").forEach((link) => {
    const targetFile = link.getAttribute("href")?.split("#")[0];
    if (targetFile && targetFile === currentFile) link.setAttribute("aria-current", "page");
  });

  let frameRequested = false;
  function updateScrollState() {
    const scrollable = document.documentElement.scrollHeight - globalThis.innerHeight;
    const amount = scrollable > 0 ? Math.min(1, Math.max(0, globalThis.scrollY / scrollable)) : 0;
    if (progress) progress.style.transform = "scaleX(" + amount + ")";
    if (header) header.classList.toggle("is-scrolled", globalThis.scrollY > 12);
    frameRequested = false;
  }
  globalThis.addEventListener(
    "scroll",
    () => {
      if (frameRequested) return;
      frameRequested = true;
      globalThis.requestAnimationFrame(updateScrollState);
    },
    { passive: true },
  );
  updateScrollState();

  const revealItems = document.querySelectorAll("[data-reveal]");
  if (
    "IntersectionObserver" in globalThis &&
    !globalThis.matchMedia("(prefers-reduced-motion: reduce)").matches
  ) {
    const observer = new globalThis.IntersectionObserver(
      (entries, currentObserver) => {
        entries.forEach((entry) => {
          if (!entry.isIntersecting) return;
          entry.target.classList.add("is-visible");
          currentObserver.unobserve(entry.target);
        });
      },
      { threshold: 0.12, rootMargin: "0px 0px -24px 0px" },
    );
    revealItems.forEach((item) => observer.observe(item));
  } else {
    revealItems.forEach((item) => item.classList.add("is-visible"));
  }

  async function copyCommand(button) {
    const label = button.querySelector("[data-en]");
    const originalText = label.dataset[language] || label.textContent;
    const command = button.dataset.copy.replace(/&#10;/g, "\n");

    try {
      await globalThis.navigator.clipboard.writeText(command);
      label.textContent = language === "en" ? "Copied" : "已复制";
      globalThis.setTimeout(() => {
        label.textContent = originalText;
      }, 1800);
    } catch {
      label.textContent = language === "en" ? "Select to copy" : "请手动复制";
    }
  }

  document.querySelectorAll("[data-copy]").forEach((button) => {
    button.addEventListener("click", () => {
      void copyCommand(button);
    });
  });

  document.querySelectorAll("[data-year]").forEach((element) => {
    element.textContent = String(new Date().getFullYear());
  });

  document.querySelectorAll(".faq-item").forEach((item) => {
    item.addEventListener("toggle", () => {
      if (!item.open) return;
      document.querySelectorAll(".faq-item").forEach((other) => {
        if (other !== item) other.open = false;
      });
    });
  });

  applyPageLanguage();
  globalThis.requestAnimationFrame(() => body.classList.add("is-ready"));
}

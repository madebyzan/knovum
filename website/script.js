(() => {
  const root = document.documentElement;
  const body = document.body;
  const languageButton = document.querySelector(".language-toggle");
  const menuButton = document.querySelector(".menu-toggle");
  const navigation = document.querySelector(".primary-nav");
  const header = document.querySelector(".site-header");
  const progress = document.querySelector(".reading-progress span");
  const metaDescription = document.querySelector('meta[name="description"]');

  const pageMeta = {
    en: {
      home: ["Knovum — Bring your knowledge together.", "Knovum is a local-first workspace for gathering, connecting, and creating with your own knowledge."],
      product: ["Product — Knovum", "Explore the Knovum workspace: Spaces, source-grounded answers, local search, structured knowledge, and traceable creation."],
      privacy: ["Privacy — Knovum", "Understand how Knovum keeps your knowledge local and makes cloud AI use explicit."],
      download: ["Download — Knovum", "Download Knovum for macOS and Windows, or build it from source."],
      about: ["About — Knovum", "Learn about Knovum's product philosophy, open-source foundations, and community."]
    },
    zh: {
      home: ["Knovum — 让知识汇聚", "Knovum 是一款本地优先的知识工作空间，用于收集资料、连接想法，并从自己的知识中继续创造。"],
      product: ["产品 — Knovum", "探索 Knovum 工作空间：Spaces、引用溯源、本地搜索、结构化知识与可追溯创作。"],
      privacy: ["隐私 — Knovum", "了解 Knovum 如何让知识保存在本地，并明确控制云端 AI 的使用。"],
      download: ["下载 — Knovum", "获取适用于 macOS 和 Windows 的 Knovum，或从源代码构建。"],
      about: ["关于 — Knovum", "了解 Knovum 的产品理念、开源基础与社区。"]
    }
  };

  const page = body.dataset.page || "home";
  const safeStorage = {
    get(key) { try { return window.localStorage.getItem(key); } catch { return null; } },
    set(key, value) { try { window.localStorage.setItem(key, value); } catch { /* Storage can be disabled. */ } }
  };
  let language = safeStorage.get("knovum-language") === "zh" ? "zh" : "en";

  function updateMenuLabel() {
    if (!menuButton) return;
    const isOpen = menuButton.getAttribute("aria-expanded") === "true";
    menuButton.setAttribute("aria-label", language === "en"
      ? (isOpen ? "Close navigation" : "Open navigation")
      : (isOpen ? "关闭导航" : "打开导航"));
  }

  function setLanguage(next) {
    language = next;
    safeStorage.set("knovum-language", language);
    root.lang = language === "zh" ? "zh-CN" : "en";

    document.querySelectorAll("[data-en][data-zh]").forEach((element) => {
      element.textContent = element.dataset[language];
    });

    const [title, description] = pageMeta[language][page] || pageMeta[language].home;
    document.title = title;
    if (metaDescription) metaDescription.setAttribute("content", description);
    if (languageButton) {
      languageButton.textContent = language === "en" ? "中文" : "EN";
      languageButton.setAttribute("aria-label", language === "en" ? "Switch to Chinese" : "切换到英文");
    }
    updateMenuLabel();
  }

  if (languageButton) {
    languageButton.addEventListener("click", () => setLanguage(language === "en" ? "zh" : "en"));
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
    document.addEventListener("keydown", (event) => { if (event.key === "Escape") closeMenu(); });
  }

  const currentFile = window.location.pathname.split("/").pop() || "index.html";
  document.querySelectorAll(".primary-nav a[href]").forEach((link) => {
    const targetFile = link.getAttribute("href")?.split("#")[0];
    if (targetFile && targetFile === currentFile) link.setAttribute("aria-current", "page");
  });

  let frameRequested = false;
  function updateScrollState() {
    const scrollable = document.documentElement.scrollHeight - window.innerHeight;
    const amount = scrollable > 0 ? Math.min(1, Math.max(0, window.scrollY / scrollable)) : 0;
    if (progress) progress.style.transform = `scaleX(${amount})`;
    if (header) header.classList.toggle("is-scrolled", window.scrollY > 12);
    frameRequested = false;
  }
  window.addEventListener("scroll", () => {
    if (frameRequested) return;
    frameRequested = true;
    window.requestAnimationFrame(updateScrollState);
  }, { passive: true });
  updateScrollState();

  const revealItems = document.querySelectorAll("[data-reveal]");
  if ("IntersectionObserver" in window && !window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
    const observer = new IntersectionObserver((entries, currentObserver) => {
      entries.forEach((entry) => {
        if (!entry.isIntersecting) return;
        entry.target.classList.add("is-visible");
        currentObserver.unobserve(entry.target);
      });
    }, { threshold: .12, rootMargin: "0px 0px -24px 0px" });
    revealItems.forEach((item) => observer.observe(item));
  } else {
    revealItems.forEach((item) => item.classList.add("is-visible"));
  }

  document.querySelectorAll("[data-copy]").forEach((button) => {
    button.addEventListener("click", async () => {
      const initialText = button.querySelector("[data-en]")?.dataset[language] || button.textContent.trim();
      const command = button.dataset.copy.replace(/&#10;/g, "\n");
      try {
        if (!navigator.clipboard?.writeText) throw new Error("Clipboard API unavailable");
        await navigator.clipboard.writeText(command);
        const label = button.querySelector("[data-en]");
        if (label) label.textContent = language === "en" ? "Copied" : "已复制";
        else button.textContent = language === "en" ? "Copied" : "已复制";
        window.setTimeout(() => {
          if (label) label.textContent = initialText;
          else button.textContent = initialText;
        }, 1800);
      } catch {
        const label = button.querySelector("[data-en]");
        if (label) label.textContent = language === "en" ? "Select to copy" : "请手动复制";
      }
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

  setLanguage(language);
  requestAnimationFrame(() => body.classList.add("is-ready"));
})();

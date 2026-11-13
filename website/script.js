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
      home: ["Knovum | Local-First Knowledge Management & AI Research Workspace", "Knovum is a free, open-source local-first knowledge management workspace for desktop research, source-grounded AI answers, semantic search, and creation."],
      product: ["Knovum Product | Infinite Canvas, Research & Source-Cited AI", "Explore an infinite canvas for research, semantic search across notes and PDFs, source-cited AI answers, structured knowledge, and traceable creation in Knovum."],
      privacy: ["Private Knowledge Base | Local-First Desktop App — Knovum", "Learn how Knovum stores your knowledge locally, supports on-device search and processing, protects API credentials, and makes cloud AI data transfers explicit."],
      download: ["Download Knovum | Free Open-Source macOS & Windows App", "Download the free, open-source Knovum knowledge workspace for macOS and Windows. Check supported versions, installation notes, and build instructions."],
      about: ["About Knovum | Open-Source Knowledge Workspace", "Learn about Knovum's local-first philosophy, open-source foundations, technical architecture, and community."]
    },
    zh: {
      home: ["Knovum｜本地优先知识管理与 AI 研究工作空间", "Knovum 是免费开源的本地优先知识管理软件，面向桌面研究、可溯源 AI 问答、语义搜索与知识创作。"],
      product: ["Knovum 产品｜无限画布、研究工作空间与来源引用 AI", "了解 Knovum 无限画布研究工作空间，探索笔记与 PDF 语义搜索、来源引用 AI 问答、结构化知识和可追溯创作。"],
      privacy: ["隐私知识库｜本地优先桌面应用 — Knovum", "了解 Knovum 如何在本地保存知识、支持设备端搜索与处理、保护 API 密钥，并明确控制云端 AI 数据传输。"],
      download: ["下载 Knovum｜免费开源 macOS 与 Windows 知识管理软件", "下载适用于 macOS 和 Windows 的免费开源 Knovum 知识工作空间，查看系统要求、安装说明和源代码构建方法。"],
      about: ["关于 Knovum｜开源知识工作空间", "了解 Knovum 的本地优先理念、开源基础、技术架构与社区。"]
    }
  };

  const page = body.dataset.page || "home";
  const safeStorage = {
    get(key) {
      try {
        return globalThis.localStorage.getItem(key);
      } catch {
        return null;
      }
    },
    set(key, value) {
      try {
        globalThis.localStorage.setItem(key, value);
      } catch {
        // Storage can be disabled by the browser.
      }
    }
  };
  const savedLanguage = safeStorage.get("knovum-language");
  const pageLanguage = body.dataset.locale === "zh" ? "zh" : "en";
  let language = savedLanguage === "zh" || savedLanguage === "en" ? savedLanguage : pageLanguage;

  function updateMenuLabel() {
    if (!menuButton) return;
    const isOpen = menuButton.getAttribute("aria-expanded") === "true";
    const label = language === "en"
      ? (isOpen ? "Close navigation" : "Open navigation")
      : (isOpen ? "关闭导航" : "打开导航");
    menuButton.setAttribute("aria-label", label);
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
  globalThis.addEventListener("scroll", () => {
    if (frameRequested) return;
    frameRequested = true;
    globalThis.requestAnimationFrame(updateScrollState);
  }, { passive: true });
  updateScrollState();

  const revealItems = document.querySelectorAll("[data-reveal]");
  if ("IntersectionObserver" in globalThis && !globalThis.matchMedia("(prefers-reduced-motion: reduce)").matches) {
    const observer = new globalThis.IntersectionObserver((entries, currentObserver) => {
      entries.forEach((entry) => {
        if (!entry.isIntersecting) return;
        entry.target.classList.add("is-visible");
        currentObserver.unobserve(entry.target);
      });
    }, { threshold: 0.12, rootMargin: "0px 0px -24px 0px" });
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

  setLanguage(language);
  globalThis.requestAnimationFrame(() => body.classList.add("is-ready"));
})();

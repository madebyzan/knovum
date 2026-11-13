(() => {
  const root = document.documentElement;
  const languageButton = document.querySelector(".language-toggle");
  const menuButton = document.querySelector(".menu-toggle");
  const navigation = document.querySelector(".primary-nav");
  const header = document.querySelector(".site-header");
  const metaDescription = document.querySelector('meta[name="description"]');

  const pageMeta = {
    en: {
      title: "Knovum — A place for everything you're learning.",
      description: "Knovum is a local-first knowledge workspace for collecting sources, connecting ideas, asking grounded questions, and creating work you can trace back."
    },
    zh: {
      title: "Knovum — 为你的知识，留一处空间。",
      description: "Knovum 是一款本地优先的知识工作空间，用于收集资料、连接想法、提出有依据的问题，并创作可追溯的成果。"
    }
  };

  let language = "en";

  function updateMenuLabel() {
    const isOpen = menuButton.getAttribute("aria-expanded") === "true";
    const label = language === "en"
      ? (isOpen ? "Close navigation" : "Open navigation")
      : (isOpen ? "关闭导航" : "打开导航");
    menuButton.setAttribute("aria-label", label);
  }

  function setLanguage(nextLanguage) {
    language = nextLanguage;
    root.lang = language === "zh" ? "zh-CN" : "en";
    document.title = pageMeta[language].title;
    metaDescription.setAttribute("content", pageMeta[language].description);

    document.querySelectorAll("[data-en][data-zh]").forEach((element) => {
      element.textContent = element.dataset[language];
    });

    languageButton.textContent = language === "en" ? "中文" : "EN";
    languageButton.setAttribute("aria-label", language === "en" ? "Switch to Chinese" : "切换到英文");
    updateMenuLabel();
  }

  languageButton.addEventListener("click", () => {
    setLanguage(language === "en" ? "zh" : "en");
  });

  function closeMenu() {
    navigation.classList.remove("is-open");
    menuButton.setAttribute("aria-expanded", "false");
    updateMenuLabel();
  }

  menuButton.addEventListener("click", () => {
    const isOpen = menuButton.getAttribute("aria-expanded") === "true";
    menuButton.setAttribute("aria-expanded", String(!isOpen));
    navigation.classList.toggle("is-open", !isOpen);
    updateMenuLabel();
  });

  navigation.querySelectorAll("a").forEach((link) => {
    link.addEventListener("click", closeMenu);
  });

  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") closeMenu();
  });

  let scrollTicking = false;
  window.addEventListener("scroll", () => {
    if (scrollTicking) return;
    scrollTicking = true;
    window.requestAnimationFrame(() => {
      header.classList.toggle("is-scrolled", window.scrollY > 12);
      scrollTicking = false;
    });
  }, { passive: true });

  const revealElements = document.querySelectorAll(".reveal");
  if ("IntersectionObserver" in window && !window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
    const observer = new IntersectionObserver((entries, activeObserver) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          entry.target.classList.add("is-visible");
          activeObserver.unobserve(entry.target);
        }
      });
    }, { threshold: 0.12 });
    revealElements.forEach((element) => observer.observe(element));
  } else {
    revealElements.forEach((element) => element.classList.add("is-visible"));
  }

  document.getElementById("year").textContent = String(new Date().getFullYear());

  document.querySelectorAll(".faq-item").forEach((item) => {
    item.addEventListener("toggle", () => {
      if (!item.open) return;
      document.querySelectorAll(".faq-item").forEach((otherItem) => {
        if (otherItem !== item) otherItem.open = false;
      });
    });
  });

  setLanguage("en");
})();

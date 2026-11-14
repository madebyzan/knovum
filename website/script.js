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

    document.querySelectorAll("[data-en-label][data-zh-label]").forEach((element) => {
      if (element.hasAttribute("data-carousel-toggle")) return;
      element.setAttribute(
        "aria-label",
        localizedText(element.dataset.enLabel, element.dataset.zhLabel),
      );
    });

    document.querySelectorAll("[data-carousel-toggle]").forEach((element) => {
      const isPaused = element.dataset.paused === "true";
      const label = isPaused
        ? localizedText("Resume automatic playback", "恢复自动播放")
        : localizedText("Pause automatic playback", "暂停自动播放");
      element.setAttribute("aria-label", label);

      const labelElement = element.querySelector("span");
      if (labelElement) {
        labelElement.textContent = isPaused
          ? localizedText("Play", "播放")
          : localizedText("Pause", "暂停");
      }
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

    document.addEventListener("pointerdown", (event) => {
      if (menuButton.getAttribute("aria-expanded") === "true" && header && !header.contains(event.target)) {
        closeMenu();
      }
      const openDropdown = navigation.querySelector(".nav-menu[open]");
      if (openDropdown && !openDropdown.contains(event.target)) openDropdown.removeAttribute("open");
    });

    document.addEventListener("keydown", (event) => {
      if (event.key !== "Escape") return;
      const wasOpen = menuButton.getAttribute("aria-expanded") === "true";
      closeMenu();
      if (wasOpen) menuButton.focus();
    });

    const desktopLayout = globalThis.matchMedia("(min-width: 761px)");
    const resetMenuForDesktop = (event) => {
      if (event.matches) closeMenu();
    };
    if (desktopLayout.addEventListener) {
      desktopLayout.addEventListener("change", resetMenuForDesktop);
    } else {
      desktopLayout.addListener(resetMenuForDesktop);
    }
  }

  const normalizePath = (pathname) => pathname.replace(/index\.html$/, "").replace(/\/$/, "");
  document.querySelectorAll(".primary-nav a[href]").forEach((link) => {
    const href = link.getAttribute("href");
    if (!href || href.startsWith("#")) return;
    const target = new URL(href, globalThis.location.href);
    if (target.origin !== globalThis.location.origin) return;
    const targetPath = normalizePath(target.pathname);
    const currentPath = normalizePath(globalThis.location.pathname);
    if (targetPath === currentPath) {
      link.setAttribute("aria-current", "page");
    } else if (targetPath.endsWith("/guides") && currentPath.startsWith(targetPath + "/")) {
      link.setAttribute("aria-current", "location");
    } else {
      link.removeAttribute("aria-current");
    }
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
  globalThis.addEventListener("resize", updateScrollState, { passive: true });

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

  const imageCarouselRefreshers = new WeakMap();

  const localizedText = (english, chinese) => (language === "en" ? english : chinese);

  function setupImageCarousels() {
    document.querySelectorAll("[data-image-carousel]").forEach((carousel) => {
      const slides = [...carousel.querySelectorAll(".image-carousel__slide")];
      const previousButton = carousel.querySelector("[data-carousel-prev]");
      const nextButton = carousel.querySelector("[data-carousel-next]");
      const toggleButton = carousel.querySelector("[data-carousel-toggle]");
      const counter = carousel.querySelector("[data-carousel-counter]");
      const reducedMotion = globalThis.matchMedia("(prefers-reduced-motion: reduce)");
      let activeSlide = null;
      let intervalId = null;
      let isPausedByUser = reducedMotion.matches;
      let isHovered = false;
      let hasFocus = false;
      let isInViewport = true;

      const availableSlides = () =>
        slides.filter((slide) => !slide.hidden && slide.isConnected);

      const setToggleLabel = () => {
        if (!toggleButton) return;
        const label = isPausedByUser
          ? localizedText("Resume automatic playback", "恢复自动播放")
          : localizedText("Pause automatic playback", "暂停自动播放");

        toggleButton.dataset.paused = String(isPausedByUser);
        toggleButton.setAttribute("aria-pressed", String(!isPausedByUser));
        toggleButton.setAttribute("aria-label", label);

        const labelElement = toggleButton.querySelector("span");
        if (labelElement) {
          labelElement.textContent = isPausedByUser
            ? localizedText("Play", "播放")
            : localizedText("Pause", "暂停");
        }
      };

      const refresh = () => {
        const visibleSlides = availableSlides();
        if (visibleSlides.length === 0) {
          carousel.classList.remove("is-ready");
          return;
        }

        if (!visibleSlides.includes(activeSlide)) activeSlide = visibleSlides[0];

        visibleSlides.forEach((slide) => {
          const isActive = slide === activeSlide;
          slide.classList.toggle("is-active", isActive);
          slide.setAttribute("aria-hidden", String(!isActive));
        });

        carousel.classList.add("is-ready");
        if (counter) {
          const index = visibleSlides.indexOf(activeSlide) + 1;
          counter.textContent =
            String(index).padStart(2, "0") +
            " / " +
            String(visibleSlides.length).padStart(2, "0");
        }

        const hasMultipleSlides = visibleSlides.length > 1;
        if (previousButton) previousButton.hidden = !hasMultipleSlides;
        if (nextButton) nextButton.hidden = !hasMultipleSlides;
        if (toggleButton) {
          toggleButton.hidden = !hasMultipleSlides || reducedMotion.matches;
        }
      };

      const showSlide = (nextIndex) => {
        const visibleSlides = availableSlides();
        if (visibleSlides.length === 0) return;

        const slideCount = visibleSlides.length;
        const targetIndex = ((nextIndex % slideCount) + slideCount) % slideCount;
        activeSlide = visibleSlides[targetIndex];
        refresh();
      };

      const stopAutoplay = () => {
        if (intervalId === null) return;
        globalThis.clearInterval(intervalId);
        intervalId = null;
      };

      const syncAutoplay = () => {
        stopAutoplay();
        if (
          isPausedByUser ||
          isHovered ||
          hasFocus ||
          !isInViewport ||
          document.visibilityState === "hidden" ||
          reducedMotion.matches ||
          availableSlides().length < 2
        ) {
          return;
        }

        intervalId = globalThis.setInterval(() => {
          const visibleSlides = availableSlides();
          showSlide(visibleSlides.indexOf(activeSlide) + 1);
        }, 5600);
      };

      previousButton?.addEventListener("click", () => {
        const visibleSlides = availableSlides();
        showSlide(visibleSlides.indexOf(activeSlide) - 1);
        syncAutoplay();
      });

      nextButton?.addEventListener("click", () => {
        const visibleSlides = availableSlides();
        showSlide(visibleSlides.indexOf(activeSlide) + 1);
        syncAutoplay();
      });

      toggleButton?.addEventListener("click", () => {
        isPausedByUser = !isPausedByUser;
        setToggleLabel();
        syncAutoplay();
      });

      carousel.addEventListener("mouseenter", () => {
        isHovered = true;
        syncAutoplay();
      });

      carousel.addEventListener("mouseleave", () => {
        isHovered = false;
        syncAutoplay();
      });

      carousel.addEventListener("focusin", () => {
        hasFocus = true;
        syncAutoplay();
      });

      carousel.addEventListener("focusout", (event) => {
        hasFocus = Boolean(event.relatedTarget && carousel.contains(event.relatedTarget));
        syncAutoplay();
      });

      document.addEventListener("visibilitychange", syncAutoplay);

      if (reducedMotion.addEventListener) {
        reducedMotion.addEventListener("change", () => {
          if (reducedMotion.matches) isPausedByUser = true;
          setToggleLabel();
          refresh();
          syncAutoplay();
        });
      }

      if ("IntersectionObserver" in globalThis) {
        const observer = new globalThis.IntersectionObserver(([entry]) => {
          isInViewport = entry.isIntersecting;
          syncAutoplay();
        });
        observer.observe(carousel);
      }

      imageCarouselRefreshers.set(carousel, () => {
        refresh();
        syncAutoplay();
      });

      setToggleLabel();
      refresh();
      syncAutoplay();
    });
  }

  setupImageCarousels();

  function loadOptionalImages() {
    document.querySelectorAll("[data-optional-image]").forEach((figure) => {
      const image = figure.querySelector("img");
      const source = figure.dataset.optionalImage;
      if (!image || !source) return;

      image.addEventListener(
        "load",
        () => {
          figure.hidden = false;
          const carousel = figure.closest("[data-image-carousel]");
          if (carousel) imageCarouselRefreshers.get(carousel)?.();
        },
        { once: true },
      );
      image.addEventListener(
        "error",
        () => {
          const carousel = figure.closest("[data-image-carousel]");
          figure.remove();
          if (carousel) imageCarouselRefreshers.get(carousel)?.();
        },
        { once: true },
      );
      image.loading = "eager";
      image.src = source;
    });

    if (language === "zh") {
      document.querySelectorAll("img[data-localized-src]").forEach((image) => {
        const localizedImage = new Image();
        localizedImage.decoding = "async";
        localizedImage.addEventListener(
          "load",
          () => {
            image.src = localizedImage.src;
          },
          { once: true },
        );
        localizedImage.src = image.dataset.localizedSrc;
      });
    }
  }

  loadOptionalImages();

  async function updateReleaseDownloads() {
    const links = [...document.querySelectorAll("[data-release-asset]")];
    if (!links.length || typeof globalThis.fetch !== "function") return;

    const latestReleaseUrl = "https://github.com/madebyzan/knovum/releases/latest";
    const assetPatterns = {
      "macos-arm64": /_aarch64\.dmg$/i,
      "macos-x64": /_x64\.dmg$/i,
      "windows-x64": /_x64-setup\.exe$/i,
    };
    const controller = new globalThis.AbortController();
    const timeout = globalThis.setTimeout(() => controller.abort(), 5000);

    try {
      const response = await globalThis.fetch(
        "https://api.github.com/repos/madebyzan/knovum/releases/latest",
        {
          headers: { Accept: "application/vnd.github+json" },
          signal: controller.signal,
        },
      );
      if (!response.ok) throw new Error("Latest release lookup failed");

      const release = await response.json();
      const assets = Array.isArray(release.assets) ? release.assets : [];

      links.forEach((link) => {
        const pattern = assetPatterns[link.dataset.releaseAsset];
        const asset = pattern && assets.find((candidate) => pattern.test(candidate.name || ""));
        link.href = asset?.browser_download_url || release.html_url || latestReleaseUrl;
        link.dataset.releaseReady = asset?.browser_download_url ? "true" : "false";
      });

      document.querySelectorAll("[data-release-version]").forEach((element) => {
        element.textContent = release.tag_name || "—";
      });
    } catch {
      links.forEach((link) => {
        link.href = latestReleaseUrl;
        link.dataset.releaseReady = "false";
      });
    } finally {
      globalThis.clearTimeout(timeout);
    }
  }

  void updateReleaseDownloads();

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
